use std::{collections::HashSet, ffi::c_void, sync::{Arc, Mutex, RwLock, atomic::AtomicUsize}};
pub static ALLOCATION_COUNT:std::sync::atomic::AtomicUsize = AtomicUsize::new(0);
pub static FREE_COUNT:std::sync::atomic::AtomicUsize = AtomicUsize::new(0);
#[macro_export]
macro_rules! make_traceble {
    ($y: ident,$( $x:ident ),*) => {
            impl crate::imaglib::utils::Trace for $y{
            fn trace(&self, reached:&mut HashSet<(*const c_void, &'static str))->Vec<GenericCyclicPtr>{
            if reached.contains(&(self as *const Self as *const c_void, std::any::type_name::<Self>())){
                return Vec::new();
            }
            reached.insert((self as *const Self as *const c_void, std::any::type_name::<Self>()));
                let mut out = Vec::new();
                $(
                    {
                        out.push(Traceable::trace(&self.$x, reached));
                    }
                )*
                out.to_iter().flatten().collect()
            }
        }
    };
}
#[macro_export]
macro_rules! make_traceble_generic {
    ($y:ident,$( $x:ident ),*) => {
            impl<T:Traceable+'static> crate::imaglib::utils::Traceable for $y<T>{
            fn trace(&self, reached:&mut HashSet<(*const c_void, &'static str)>)->Vec<GenericCyclicPtr>{
           // println!("{:#?}", std::any::type_name::<Self>());
            if reached.contains(&(self as *const Self as *const c_void, std::any::type_name::<Self>())){
                return Vec::new();
            }
            reached.insert((self as *const Self as *const c_void,std::any::type_name::<Self>()));
                let mut out = Vec::new();
                $(
                    {
                        //println!("going to:{}",std::any::type_name_of_val(&self.$x));
                        out.push(self.$x.trace(reached));
                    }
                )*
                out.into_iter().flatten().collect()
            }
        }
    };
}
pub struct ControlBlock{
    count :Mutex<usize>,
    done:Mutex<bool>
}   
pub trait Traceable{
    fn trace(&self, reached:&mut HashSet<(*const c_void, &'static str)>)->Vec<GenericCyclicPtr>{
        if reached.contains(&(self as *const Self as *const c_void, std::any::type_name::<Self>())){
            return Vec::new();
        }
        reached.insert((self as *const Self as *const c_void, std::any::type_name::<Self>()));
        Vec::new()
    } 
    fn nameof(&self)->&str{
        std::any::type_name::<Self>()
    }
}
impl Traceable for i8{

}
impl Traceable for i16{

}
impl Traceable for i32{

}
impl Traceable for i64{

}
impl Traceable for u8{

}
impl Traceable for u16{

}
impl Traceable for u32{

}
impl Traceable for u64{

}
impl Traceable for bool{}
impl Traceable for String{}
impl Traceable for char{}
impl<T:Traceable> Traceable for Vec<T>{
    fn trace(&self, reached:&mut HashSet<(*const c_void,&'static str)>)->Vec<GenericCyclicPtr> {
        if reached.contains(&(self as *const Self as *const c_void, std::any::type_name::<Self>())){
            return Vec::new();
        }
        reached.insert((self as *const Self as *const c_void, std::any::type_name::<Self>()));
        let mut out = Vec::new();
        for i in self{
            let  t = i.trace(reached);
            for j in t{
                out.push(j)
            }
        }
        out
    }
}
impl <T:Traceable> Traceable for Box<T>{

}
impl<T:Traceable> Traceable for Option<T>{
    fn trace(&self,reached:&mut HashSet<(*const c_void, &'static str)>)->Vec<GenericCyclicPtr> {
        if reached.contains(&(self as *const Self as *const c_void, std::any::type_name::<Self>())){
            return Vec::new();
        }
        reached.insert((self as *const Self as *const c_void, std::any::type_name::<Self>()));
        let mut out = Vec::new();
        for i in self.iter(){
            let  t = i.trace(reached);
            for j in t{
                out.push(j)
            }
            break;
  
        }
        out
    }
}
impl<T:Traceable> Traceable for RwLock<T>{
    fn trace(&self,reached:&mut HashSet<(*const c_void, &'static str)>)->Vec<GenericCyclicPtr> {
        if reached.contains(&(self as *const Self as *const c_void, std::any::type_name::<Self>())){
            return Vec::new();
        }
        reached.insert((self as *const Self as *const c_void, std::any::type_name::<Self>()));
        let s = self.read().unwrap();
        s.trace(reached)
    }
}
pub struct GenericCyclicPtr{
    ptr:*const dyn Traceable, 
}
pub struct CyclicPtr<T:Traceable>{
    ptr:*const T, 
    control_block:Arc<ControlBlock>,
}
impl<T:Traceable> Drop for CyclicPtr<T>{
    fn drop(&mut self) {
        let mut done = self.control_block.done.lock().unwrap();
        if *done{
            return;
        }
        if self.ptr.is_null(){
            return;
        }
        *done = true;
        drop(done);
        unsafe{
            let block = &*self.control_block;
            let mut count = block.count.lock().unwrap();
            *count -= 1;
            let cycle_count = trace(&*self.ptr, &mut HashSet::new(), self.ptr as *const c_void);
            if *count <cycle_count+1{
                println!("freed:{:#?}",self.ptr);
                FREE_COUNT.fetch_add(1,std::sync::atomic::Ordering::Acquire);
                drop(count);
                _ =  Box::from_raw(self.ptr as *mut T);

            }
        }

    }
}
impl <T:Traceable> Clone for CyclicPtr<T>{
    fn clone(&self) -> Self {
        let mut count = self.control_block.count.lock().unwrap();
        *count+=1;
        drop(count);
        Self { ptr: self.ptr.clone(), control_block: self.control_block.clone() }
    }
}
impl <T:Traceable+'static> Traceable for CyclicPtr<T>{
    fn trace(&self,reached:&mut HashSet<(*const c_void, &'static str)>)->Vec<GenericCyclicPtr> {
        if reached.contains(&(self as *const Self as *const c_void, std::any::type_name::<Self>())){
            return Vec::new();
        }
        reached.insert((self as *const Self as *const c_void,std::any::type_name::<Self>()));
        if self.ptr.is_null(){
            return Vec::new()
        }
        let mut out = unsafe{ (*self.ptr).trace(reached)};
        out.push(GenericCyclicPtr { ptr: self.ptr });
        out
    }
}
impl <T:Traceable> CyclicPtr<T>{
    pub fn new(to_bx:T)->Self{
        let ptr = Box::new(to_bx);
        let contr = Arc::new(ControlBlock{count:Mutex::new(1), done:Mutex::new(false)});
        let ptr2:*const T = Box::leak(ptr);
        println!("allocated:{:#?}", ptr2);
        ALLOCATION_COUNT.fetch_add(1,std::sync::atomic::Ordering::Acquire);
        Self{ptr:ptr2, control_block:contr}
    }
    pub fn get(&self)->&T{
        unsafe{
            return &*self.ptr;
        }
    }
}
pub unsafe fn trace(t:&dyn Traceable, reached:&mut HashSet<(*const c_void, &'static str)>, base:*const c_void)->usize{
    //println!("{:#?}", t.nameof());
    let values = Traceable::trace(t, reached);
    let mut count = 0;
    for i in values{
        if i.ptr as *const c_void == base{
            count+=1 ;
        }else{
            let nc = unsafe{trace(&*i.ptr, reached, base)};
            count+= nc;
        }
    }
    count
}
pub fn debug_allocations(){
    let a = ALLOCATION_COUNT.load(std::sync::atomic::Ordering::Acquire);
    let f = FREE_COUNT.load(std::sync::atomic::Ordering::Acquire);
    println!("allocation_count:{}, free_count:{}, equality?:{}", a, f, a == f);
}