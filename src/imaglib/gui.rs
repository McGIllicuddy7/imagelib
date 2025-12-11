use std::sync::{Arc, RwLock, RwLockReadGuard, Weak};
pub enum WidgetType{
    Div, Text, Button,TextInput,
}
pub enum IoEvent{
    OnPress, 
    OnHover, 
    OnRelease, 
    OnKeyInput,
}
#[allow(unused)]
pub struct Widget{
    internal:Weak<RwLock<WidgetInternal>>,
}
pub struct WidgetRef<'a>{
    _ref: RwLockReadGuard<'a, WidgetInternal>,
    _ptr:Arc<RwLock<WidgetInternal>>
}
pub struct WidgetInternal{
    pub wt:WidgetType,pub x:i32, pub y:i32, pub w:i32,pub  h:i32, pub string:String, pub on_interact:Box<dyn Fn(&mut WidgetInternal, IoEvent)>, pub children:Vec<Widget>
}
#[allow(unused)]
pub struct Gui{
    pub stack:Vec<Widget>,
    pub widgets:Vec<Arc<Widget>>, 
    pub selected:Option<Widget>
}
impl PartialEq for Widget{
    fn eq(&self, other: &Self) -> bool {
        self.internal.ptr_eq(&other.internal)
    }
}
impl Widget{
    pub fn get<'a>(&'a self)->WidgetRef<'a>{
        let p = self.internal.upgrade().unwrap();
        let p0 = p.clone();
        let p1 = p.read().unwrap();
        let out = WidgetRef { _ref: p1, _ptr: p0 };
        unsafe{std::mem::transmute(out)}
    }
}
impl WidgetInternal{

}
