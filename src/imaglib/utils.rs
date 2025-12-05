use std::{any::{TypeId, type_name}, collections::{HashMap, HashSet}, fmt::Debug, ops::{Deref, DerefMut}, os::{raw::c_void, unix::fs::PermissionsExt}, ptr::{null, slice_from_raw_parts}, sync::{Arc, Mutex, MutexGuard, RwLock, RwLockWriteGuard}};

use crate::imaglib::draw::rand;
pub trait GcPtr:Traceable+{
    fn is_root(&self)->bool;
    fn set_root(&self,is_root:bool);
    fn get_ptr(&self)->*const c_void;
}
pub trait Traceable{
    fn trace(&self, reached:&mut HashSet<(*const c_void, &'static str)>, reachable_table:&mut HashMap<*const c_void, Arc<Vec<*const dyn GcPtr>>>)->Arc<Vec<*const dyn GcPtr>>{
        if reachable_table.contains_key(&(self as *const Self  as *const c_void)){
            return reachable_table[&(self  as *const Self as *const c_void)].clone();
        }
        if reached.contains(&(self as *const Self as *const c_void,self.get_name())){
            return Arc::new(vec![]);
        }
        let out = Arc::new(Vec::new());
        reached.insert((self as *const Self as *const c_void,self.get_name()));
        reachable_table.insert(self  as *const Self as *const c_void, out.clone());
        out
    }
    fn get_name(&self)->&'static str{
        type_name::<Self>()
    }
}
#[derive(Clone)]
pub struct Allocation{
    data:* const dyn Traceable,
    is_reachable:bool,
}
struct GCData{
    ptrs:Vec<*const dyn GcPtr>,
    allocations:Vec<Allocation>, 
    allocations_since_last_gc:usize,
}
unsafe  impl Send for GCData{}

static GARBAGE:Mutex<GCData> = Mutex::new(GCData { ptrs: Vec::new(), allocations: Vec::new() , allocations_since_last_gc:0});
static GC_LOCK:Mutex<()> = Mutex::new(());
static ALLOCATION_COUNT:Mutex<usize> = Mutex::new(0);
static FREE_COUNT:Mutex<usize> = Mutex::new(0);
struct GcBox<T:Traceable+'static>{
    ptr:*const T,
    is_root:Mutex<bool>,
}
impl<T:Traceable+'static> GcPtr for GcBox<T>{
    fn is_root(&self)->bool {
        *self.is_root.lock().unwrap()
    }

    fn set_root(&self,is_root:bool) {
        *self.is_root.lock().unwrap() = is_root;
    }

    fn get_ptr(&self)->*const c_void {
        return self.ptr as *const c_void;
    }
}
impl<T:Traceable+'static> Traceable for GcBox<T>{
     fn trace(&self,reached:&mut HashSet<(*const c_void, &'static str)>, reachable_table:&mut std::collections::HashMap<*const c_void, Arc<Vec<*const dyn crate::imaglib::utils::GcPtr>>>)->Arc<Vec<*const dyn GcPtr>>{
        if reachable_table.contains_key(&(self as *const Self  as *const c_void)){
            return reachable_table[&(self  as *const Self as *const c_void)].clone();
        }
        if reached.contains(&(self as *const Self as *const c_void,self.get_name())){
            return Arc::new(vec![])
        }
        reached.insert((self as *const Self as *const c_void,self.get_name()));
        let base = unsafe{&*self.ptr}.trace(reached,reachable_table);
        let mut b = base.to_vec();
        b.push(self as *const dyn GcPtr);
        let out = Arc::new(b);
        reachable_table.insert(self as *const Self as *const c_void, out.clone());
        out
    }
}
pub struct GC<T:Traceable+'static>{
    bx:Box<GcBox<T>>
}

impl<T:Traceable+'static> GC<T>{
    pub fn new(v:T)->Self{
        //println!("created");
        let mut lock = GARBAGE.lock().unwrap();
        let _handle = GC_LOCK.lock().unwrap();
        let allocation = Box::leak(Box::new(v)) as *const T;
        lock.allocations.push(Allocation { data: allocation, is_reachable:true });
        let mut h = ALLOCATION_COUNT.lock().unwrap();
        *h+=1;
        lock.allocations_since_last_gc += 1;
        drop(h);
        let bx = GcBox{ptr:allocation, is_root:Mutex::new(true)};
        let bx2 = Box::new(bx);
        lock.ptrs.push(bx2.as_ref());
        Self { bx: bx2 }
    }
}
impl<T:Traceable+'static> Deref for GC<T>{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe{&*self.bx.ptr}
    }
}
impl<T:Traceable+'static> Drop for GC<T>{
    fn drop(&mut self) {
        let mut gb = GARBAGE.lock().unwrap();
        let mut idx = None;
        for i in 0..gb.ptrs.len(){
            if gb.ptrs[i] as *const c_void== self.bx.as_ref() as *const GcBox<T> as *const c_void{
                idx = Some(i);
            }
        }
        let Some(id) = idx else {
            return ;
        };
        gb.ptrs.remove(id);
        gb.allocations_since_last_gc+=1;
        drop(gb);
    }
}
impl <T:Traceable+'static> Clone for GC<T>{
    fn clone(&self) -> Self {
        let mut lock = GARBAGE.lock().unwrap();
        let _handle = GC_LOCK.lock().unwrap();
        let allocation = self.bx.ptr;
        let bx = GcBox{ptr:allocation, is_root:Mutex::new(true)};
        let bx2 = Box::new(bx);
        lock.ptrs.push(bx2.as_ref());
        drop(lock);
        Self { bx: bx2 }
    }
}
impl <T:Traceable+'static+Debug>  Debug for GC<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GC").field("bx", unsafe{&*self.bx.ptr}).finish()
    }
}

impl Allocation{
    pub fn free(&mut self){
        unsafe {
            let bx:Box<dyn Traceable> = Box::from_raw(self.data as *mut dyn Traceable);
            let mut h = FREE_COUNT.lock().unwrap();
            *h+=1;
             drop(h);
            drop(bx);

        }
    }
}

impl Traceable for u8{}
impl Traceable for u16{}
impl Traceable for u32{}
impl Traceable for u64{}
impl Traceable for i8{}
impl Traceable for i16{}
impl Traceable for i32{}
impl Traceable for i64{}
impl Traceable for str{}
impl Traceable for &str{}
impl Traceable for String{}
impl <T:Traceable> Traceable for Vec<T>{
    fn trace(&self, reached:&mut HashSet<(*const c_void, &'static str)>,reachable_table:&mut HashMap<*const c_void, Arc<Vec<*const dyn GcPtr>>>)->Arc<Vec<*const dyn GcPtr>> {
        if reached.contains(&(self as *const Self as *const c_void,self.get_name())){
            return Arc::new(vec![])
        }
        reached.insert((self as *const Self as *const c_void,self.get_name()));
        if reachable_table.contains_key(&(self as *const Self  as *const c_void)){
            return reachable_table[&(self  as *const Self as *const c_void)].clone();
        }
        let mut out = Vec::new();
        for i in self{
            let tmp = i.trace(reached, reachable_table);
            for j in tmp.iter(){
                out.push(*j);
            }
        }
        let t = Arc::new(out);
        reachable_table.insert(self as *const Self as *const c_void, t.clone());
        t
    }
}
impl <T:Traceable> Traceable for HashSet<T>{
    fn trace(&self, reached:&mut HashSet<(*const c_void, &'static str)>,reachable_table:&mut HashMap<*const c_void, Arc<Vec<*const dyn GcPtr>>>)->Arc<Vec<*const dyn GcPtr>> {
        if reached.contains(&(self as *const Self as *const c_void,self.get_name())){
            return Arc::new(vec![])
        }
        reached.insert((self as *const Self as *const c_void,self.get_name()));
        if reachable_table.contains_key(&(self as *const Self  as *const c_void)){
            return reachable_table[&(self  as *const Self as *const c_void)].clone();
        }

        let mut out = Vec::new();
        for i in self{
            let tmp = i.trace(reached,reachable_table);
            for j in tmp.iter(){
                out.push(*j);
            }
        }
        let tmp = Arc::new(out);
        reachable_table.insert(self as *const Self as *const c_void, tmp.clone());
        tmp
    }
}
impl <T:Traceable, U:Traceable> Traceable for HashMap<T, U>{
    fn trace(&self, reached:&mut HashSet<(*const c_void,&'static str)>,reachable_table:&mut HashMap<*const c_void, Arc<Vec<*const dyn GcPtr>>>)->Arc<Vec<*const dyn GcPtr>> {
        if reached.contains(&(self as *const Self as *const c_void,self.get_name())){
            return Arc::new(vec![])
        }
        reached.insert((self as *const Self as *const c_void,self.get_name()));
        if reachable_table.contains_key(&(self as *const Self  as *const c_void)){
            return reachable_table[&(self  as *const Self as *const c_void)].clone();
        }

        let mut out = Vec::new();
        for i in self{
            let tmp = i.1.trace(reached,reachable_table);
            for j in tmp.iter(){
                out.push(*j);
            }
            let tmp2 = i.0.trace(reached,reachable_table);
            for j in tmp2.iter(){
                out.push(*j);
            }
        }
        let tmp = Arc::new(out);
        reachable_table.insert(self as *const Self as *const c_void, tmp.clone());
        tmp
    }
}
impl <T:Traceable> Traceable for RwLock<T>{
    fn trace(&self, reached:&mut HashSet<(*const c_void, &'static str)>,reachable_table:&mut HashMap<*const c_void, Arc<Vec<*const dyn GcPtr>>>)->Arc<Vec<*const dyn GcPtr>> {
        if reached.contains(&(self as *const Self as *const c_void,self.get_name())){
            return Arc::new(vec![])
        }
        reached.insert((self as *const Self as *const c_void,self.get_name()));
        if reachable_table.contains_key(&(self as *const Self  as *const c_void)){
            return reachable_table[&(self  as *const Self as *const c_void)].clone();
        }
        let Ok(s) = self.try_read()else{
            reachable_table.insert(self as *const Self as *const c_void, Arc::new(Vec::new()));
             return Arc::new(vec![]);
        };
       let out = s.trace(reached,reachable_table);
       reachable_table.insert(self as *const Self as *const c_void, out.clone());
       out
    }
}
impl <T:Traceable> Traceable for Mutex<T>{
    fn trace(&self, reached:&mut HashSet<(*const c_void, &'static str)>,reachable_table:&mut HashMap<*const c_void, Arc<Vec<*const dyn GcPtr>>>)->Arc<Vec<*const dyn GcPtr>> {
        if reached.contains(&(self as *const Self as *const c_void,self.get_name())){
            return Arc::new(vec![]);
        }
        reached.insert((self as *const Self as *const c_void,self.get_name()));
        if reachable_table.contains_key(&(self as *const Self  as *const c_void)){
            return reachable_table[&(self  as *const Self as *const c_void)].clone();
        }
        let Ok(s) = self.try_lock() else {
            reachable_table.insert(self as *const Self as *const c_void, Arc::new(Vec::new()));
            return Arc::new(vec![]);
        };
        let out =s.trace(reached,reachable_table);
        reachable_table.insert(self as *const Self as *const c_void, out.clone());
        drop(s);
        out
    }
}
#[macro_export]
macro_rules! make_traceable{
    ( $t:ident,$( $x:ident ),* ) => {
        impl crate::imaglib::utils::Traceable for $t {
            fn trace(&self, reached:&mut std::collections::HashSet<(*const std::os::raw::c_void, &'static str)>,reachable_table:&mut std::collections::HashMap<*const std::os::raw::c_void, std::sync::Arc<Vec<*const dyn crate::imaglib::utils::GcPtr>>>)->std::sync::Arc<Vec<*const dyn crate::imaglib::utils::GcPtr>> {
                if reached.contains(&(self as *const Self as *const std::os::raw::c_void,self.get_name())){
                    return std::sync::Arc::new(vec![]);
                }
                reached.insert((self as *const Self as *const std::os::raw::c_void,self.get_name()));
                if reachable_table.contains_key(&(self as *const Self  as *const std::os::raw::c_void)){
                    return reachable_table[&(self  as *const Self as *const std::os::raw::c_void)].clone();
                }

                let mut out = Vec::new();
                {
                 $(
                    {
                        let tmp = self.$x.trace(reached,reachable_table);
                        out.reserve(tmp.len());
                        for j in tmp.iter(){
                            out.push(*j);
                        }
                    }
                )*
                }
                let tmp = std::sync::Arc::new(out);
                reachable_table.insert(self as *const Self as *const std::os::raw::c_void,tmp.clone());
                tmp
            }
        }
    };
}
#[macro_export]
macro_rules! make_traceable_generics{
    ( $t:ident,$( $x:ident ),* ) => {
        impl<T:Traceable+'static> crate::imaglib::utils::Traceable for $t<T> {
            fn trace(&self, reached:&mut std::collections::HashSet<*const std::os::raw::c_void>,reachable_table:&mut std::collections::HashMap<*const dyn crate::imaglib::utils::Traceable, std::sync::Arc<Vec<*const dyn crate::imaglib::utils::GcPtr>>>)->std::sync::Arc<Vec<&const dyn crate::imaglib::utils::GcPtr>> {
                if reachable_table.contains_key(&(self as *const Self  as *const std::os::raw::c_void)){
                if reached.contains(&(self as *const Self as *const std::os::raw::c_void)){
                    return Arc::new(vec![]);
                }
                reached.insert(self as *const Self as *const std::os::raw::c_void);
                let mut out = std::sync::Arc::new(Vec::new());
                    return reachable_table[&(self  as *const Self as *const std::os::raw::c_void)].clone();
                }

                {
                 $(
                    {
                        let tmp = self.$x.trace(reached);
                        out.reserve(tmp.len());
                        for j in tmp.iter(){
                            out.push(*j);
                        }
                    }
                )*
                }
                let tmp = std::sync::Arc::new(out);
                reachable_table.insert(self as *const Self as *const std::os::raw::c_void,tmp.clone());
                tmp
            }
        }
    };
}


#[derive(Debug)]
pub struct ObjRef<'a, T:Traceable+'static>{
    lock:MutexGuard<'a, T>
}
impl<'a, T:Traceable+'static> Deref for ObjRef<'a,T>{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.lock.deref()
    }
}
impl<'a, T:Traceable+'static> DerefMut for ObjRef<'a, T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.lock.deref_mut()
    }
}

#[derive(Clone)]
pub struct Object<T:Traceable+'static>{
    ptr:GC<Mutex<T>>,
}
impl<'a,T:Traceable+'static> Object<T> {
    pub fn new(v:T)->Self{
        Self { ptr: GC::new(Mutex::new(v)) }
    }
    pub fn get(&'a self)->ObjRef<'a,T>{
        ObjRef { lock: self.ptr.lock().unwrap() }
    }
}
impl<T:Traceable+'static> Traceable for Object<T>{
    fn trace(&self, reached:&mut HashSet<(*const c_void,&'static str)>,reachable_table:&mut std::collections::HashMap<*const c_void, Arc<Vec<*const dyn crate::imaglib::utils::GcPtr>>>)->Arc<Vec<*const dyn GcPtr>> {
        if reached.contains(&(self as *const Self as *const c_void,self.get_name())){
            return Arc::new(vec![])
        }
        reached.insert((self as *const Self as *const c_void,self.get_name()));  
        if reachable_table.contains_key(&(self as *const Self  as *const c_void)){
            return reachable_table[&(self  as *const Self as *const c_void)].clone();
        }
        let  mut out =self.ptr.trace(reached,reachable_table).to_vec();
        out.push(self.ptr.bx.as_ref() as *const dyn GcPtr);
        let tmp = Arc::new(out);
        reachable_table.insert(self as *const Self as *const c_void, tmp.clone());
        tmp
    }
}
impl<T:Traceable+'static> Traceable for Option<T>{
    fn trace(&self, reached:&mut HashSet<(*const c_void, &'static str)>,reachable_table:&mut std::collections::HashMap<*const c_void, Arc<Vec<*const dyn crate::imaglib::utils::GcPtr>>>)->Arc<Vec<*const dyn GcPtr>> {
        if reachable_table.contains_key(&(self as *const Self  as *const c_void)){
            return reachable_table[&(self  as *const Self as *const c_void)].clone();
        }
        if reached.contains(&(self as *const Self as *const c_void,self.get_name())){
            return Arc::new(vec![])
        }
        reached.insert((self as *const Self as *const c_void,self.get_name()));
        let out = match self{
            Some(v) => {
                v.trace(reached,reachable_table)
            }
            None => {
                Arc::new(Vec::new())
            }
        };
        reachable_table.insert(self as *const Self as *const c_void, out.clone());
        out
    }
}
impl<T:Traceable+'static> PartialEq for Object<T>{
    fn eq(&self, other: &Self) -> bool {
        self.ptr.bx.ptr == other.ptr.bx.ptr
    }
}
pub fn garbage_collect_h(mut _handle:MutexGuard<()>, can_not:bool){
    if can_not{
        let gb = GARBAGE.lock().unwrap();
        if gb.allocations_since_last_gc<512{
            return;
        }
    }
    let mut fs_count =0;
    loop{
        let mut gb = GARBAGE.lock().unwrap();
        gb.collect();
        let data = gb.allocations.clone();
        let old_len = data.len();
        drop(gb);
        let tmp;
        (_handle,tmp )= sweep(_handle,data);
        gb = GARBAGE.lock().unwrap();
        gb.allocations = tmp;
        gb.allocations_since_last_gc = 0;
        if gb.allocations.len() == old_len{
            break;
        }
        fs_count+=1;
        if fs_count>1{
            break;
        }
    }
}
impl GCData{
    //cargo r  17.70s user 0.06s system 97% cpu 18.124 total with reachable table
    //cargo r  25.61s user 0.03s system 98% cpu 26.037 total, without reachable table
    pub fn find_alloc(&mut self, ptr:*const c_void, allocation_table:&HashMap<*const c_void, usize>)->Option<&mut Allocation>{
        if let Some(idx) = allocation_table.get(&ptr){
            return Some(&mut self.allocations[*idx]);
        }
        None
    }
    pub fn mark(&mut self, ptr:*const dyn GcPtr, reachable_table:&HashMap<*const c_void, Arc<Vec<*const dyn GcPtr>>>, allocation_table:&HashMap<*const c_void, usize>){
        unsafe{
            let alo = self.find_alloc((*ptr).get_ptr(),
        allocation_table);
            let Some(al) = alo else {
                    return;
            };
            if al.is_reachable{
                return;
            }
            al.is_reachable = true;
            let tr =&reachable_table[&(ptr as *const c_void)];
            for i in tr.iter(){
                self.mark(*i, reachable_table,allocation_table);
            }
        }
    }
    pub fn collect(&mut self){           
        let mut reachable_table = HashMap::new();
        unsafe{
            for i in &self.ptrs{
                (**i).set_root(true);
            }

            for i in 0..self.ptrs.len(){
                let children = (*self.ptrs[i]).trace(&mut HashSet::new(),&mut reachable_table);
                for j in children.iter(){
                    if *j as *const c_void != self.ptrs[i] as *const c_void{
                        //println!("{:#?} makes {:#?} not a root", j, self.ptrs[i]);
                       (**j).set_root(false);
                    }
                }
                reachable_table.insert(self.ptrs[i] as *const c_void, children);
            }
        }
        let mut alloc_table = HashMap::new();
        for i in 0.. self.allocations.len(){
            alloc_table.insert(self.allocations[i].data as *const c_void, i);
            self.allocations[i].is_reachable = false;
        }
        for i in self.ptrs.clone(){
            unsafe{
                if (*i).is_root(){
                    self.mark(i,&reachable_table, &alloc_table);
                }
 
            }

        }
    }
}
pub fn sweep(_handle:MutexGuard<()>, mut allocations:Vec<Allocation>)->(MutexGuard<()>,Vec<Allocation>){
        unsafe {
        'restart:loop{
            for i in 0..allocations.len(){
                    if !allocations[i].is_reachable{
                        allocations[i].free();
                        //println!("freed:{:#?}", allocations[i].data);
                        allocations.remove(i);
                        continue 'restart;
                    }else{
                       // println!("{:#?} is reachable", allocations[i].data);
                    }
            }
            break;
        }
        }
        (_handle,allocations)
}
impl<T:Debug+Traceable> Debug for Object<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe{
            f.debug_struct("Object").field("ptr", &(*self.ptr.bx.ptr).lock().unwrap()).finish()
        }

    }
}
pub fn gc_collect(){
    let h = GC_LOCK.lock().unwrap();
    garbage_collect_h(h, false);
}
pub fn debug_alloc_free_counts(){
    let al = *ALLOCATION_COUNT.lock().unwrap();
    let fl = *FREE_COUNT.lock().unwrap();
    println!("allocations:{:#?}, frees:{:#?}, equal?:{:#?}", al, fl, al == fl)
}
pub fn spawn_gc_thread(){
    std::thread::spawn(||{gc_thread()});
}
static GC_THREAD_KILLED:Mutex<bool> = Mutex::new(false);
pub fn gc_thread(){
    loop{
        std::thread::sleep(std::time::Duration::from_millis(10));
        println!("collecting");
        gc_collect();
        if *GC_THREAD_KILLED.lock().unwrap(){
            break;
        }
    }
}
pub fn gc_thread_end(){
    *GC_THREAD_KILLED.lock().unwrap() = true;
}