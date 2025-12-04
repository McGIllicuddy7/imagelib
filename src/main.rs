pub mod imaglib;
use std::{collections::{HashMap, HashSet}, ffi::c_void, sync::{Mutex, RwLock}};

pub use imaglib::utils::*;
#[derive(Debug)]
pub struct Node<T:Traceable+'static>{
    value:T, 
    next:Option<CyclicPtr<RwLock<Node<T>>>>, 
}

make_traceble_generic!(Node,value, next);
impl <T:Traceable+'static> Node<T>{
    pub fn new(value:T)->Self{
        Self { value, next: None}
    }
    pub fn push(&mut self, value:Node<T>){
        if let Some(n) = self.next.as_ref(){
            let mut lock = n.get().write().unwrap();
            lock.push(value);
        }else{
            self.next = Some(CyclicPtr::new(RwLock::new(value)));
        }
    }
    pub fn last(&self)->Option<CyclicPtr<RwLock<Node<T>>>>{
        if let Some(n) = self.next.as_ref(){
            let f = n.get().read().unwrap();
            if f.next.is_some(){
                return f.last();
            }else{
                return self.next.clone();
            }
        }else{
            None
        }
    }
}
pub fn debug(){
    let l = {    
        let list = CyclicPtr::new(RwLock::new(Node::new(10)));
        let mut lock = list.get().write().unwrap();
        for i in 0..32{
            lock.push(Node::new(i));
        }
        println!("done\n");
        drop(lock);
        let f = list.get().read().unwrap().last().unwrap();
        f.get().write().unwrap().next = Some(list.clone());
        list.clone()
    };

    println!("testing 4,5,6");
}
fn main() {
    debug();
    debug_allocations();
}
