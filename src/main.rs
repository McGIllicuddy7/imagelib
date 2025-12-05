use std::thread::sleep;

use crate::imaglib::{draw::{rand, srand_time}, utils::{GC, Object, debug_alloc_free_counts, gc_collect, gc_thread_end, spawn_gc_thread}};

pub mod imaglib;
#[derive(Debug,Clone)]
pub struct Graph{
    pub value:i32, 
    pub connections:Vec<Object<Graph>>
}
make_traceable!(Graph, value, connections);

pub fn test()->Object<Graph>{
    let mut graphs = Vec::new();
    for i in 0..1000{
        graphs.push(Object::new(Graph{value:i, connections:Vec::new()}));
    }
    for i in 0..10000{
        let base = rand() as usize%graphs.len();
        let second = rand() as usize%graphs.len();
        graphs[base].get().connections.push(graphs[second].clone());
        let tmp = Object::new(i);
        println!("{:#?}", tmp);
    }
    return graphs[0].clone();
  
}
pub fn main(){
    spawn_gc_thread();
    srand_time();
    {
        let s = test();
        for i in &s.get().connections{
            println!("{:#?}",i.get().value);
        }
        println!("done");
        debug_alloc_free_counts();
    }
    sleep(std::time::Duration::from_millis(500));
    gc_thread_end();
    debug_alloc_free_counts();
}