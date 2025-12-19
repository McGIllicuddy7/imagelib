use std::{collections::HashMap, error::Error, ops::{Deref, DerefMut}, sync::{Arc, Mutex, MutexGuard, Weak}};

use crate::{imaglib::{Throwable, Throws, draw::{H, Image, colors::{self, BLACK, GREY, WHITE}}}, throw};
#[derive(Clone,Debug)]
pub enum WidgetType{
    None, Button, Text, Div, TextInput,Image,
}
#[derive(Debug,Clone, Copy)]
pub struct Bounds{
   pub x:i32, pub y:i32, pub w:i32, pub h:i32
}
#[derive(Clone)]
pub struct WidgetData{
    pub w_type:WidgetType,
    pub vertical:bool,
    pub x:i32, 
    pub y:i32, 
    pub w:i32, 
    pub h:i32,
    pub h_padding:i32, 
    pub w_padding:i32,
    pub text_height:i32,
    pub children:Vec<Widget>,
    pub text:String,
    pub name:String,
    pub on_update:Option<Arc<dyn Fn(&mut Gui, &mut WidgetData, GuiEvent)>>,
}

pub struct WeakNot<T>{
    v:Arc<T>
}
impl<T> Clone for WeakNot<T>{
    fn clone(&self) -> Self {
        Self { v: self.v.clone() }
    }
}
impl<T> WeakNot<T>{
    pub fn from(tmp:&Arc<T>)->Self{
        Self { v: tmp.clone() }
    }
    pub fn upgrade(&self)->Option<Arc<T>>{
        Some(self.v.clone())
    }
}
#[derive(Clone)]
pub struct Widget{
    v:WeakNot<Mutex<WidgetData>>
}
pub struct WidgetRef<'a>{
    v:MutexGuard<'a, WidgetData>, 
    _arc:Arc<Mutex<WidgetData>>,
}
pub enum  GuiEvent {
   KeyBoardInput{key:char},
   Pressed, Released,Nothing,
}
pub struct GuiInput{
    pub events:Vec<GuiEvent>,
    pub mouse_x:i32, 
    pub mouse_y:i32,
    pub is_mouse_down:bool,
}

pub struct Gui{
    pub widgets:HashMap<String,Arc<Mutex<WidgetData>>>,
    pub current_selected:Option<String>,
    pub input:GuiInput,
    pub destroy_queue:Vec<String>,
    pub root:Arc<Mutex<WidgetData>>,
}

impl<'a> Deref for WidgetRef<'a>{
    type Target = WidgetData;
    fn deref(&self) -> &Self::Target {
       &*self.v
    }
}
impl<'a> DerefMut for WidgetRef<'a>{
    fn deref_mut(&mut self) -> &mut Self::Target {
       &mut *self.v
    }
}

impl Widget{
    pub fn get<'a>(&'a self)->WidgetRef<'a>{
        let ac = self.v.upgrade().unwrap();
        let tmp = ac.lock().unwrap();
        WidgetRef { v: unsafe{std::mem::transmute(tmp)}, _arc:ac }
    }
    pub fn text(&self,gui:&mut Gui, text:String, name:String)->Throwable<Widget>{
        let w = WidgetData{
            w_type:WidgetType::Text, vertical:true, x:0, y:0, w:0, h:0, h_padding:4, w_padding:4, text_height:12, children:Vec::new(), text:text, name:name.clone(), on_update:None,
        };
        let mut s = self.get();
        let v = Arc::new(Mutex::new(w));
        let wid = Widget{v:WeakNot::from(&v)};
        s.children.push(wid.clone());
        gui.widgets.insert(name, v);
        Ok(wid) 
    }
    pub fn button<T:Fn(&mut Gui, &mut WidgetData,GuiEvent)+'static>(&self,gui:&mut Gui, on_update:T, name:String)->Throwable<Widget>{
        let w = WidgetData{
            w_type:WidgetType::Button, vertical:true, x:0, y:0, w:0, h:0, h_padding:4, w_padding:4, text_height:12, children:Vec::new(), text:String::new(), name:name.clone(), on_update:Some(Arc::new(on_update))
        };
        let mut s = self.get();
        let v = Arc::new(Mutex::new(w));
        let wid = Widget{v:WeakNot::from(&v)};
        s.children.push(wid.clone());
        gui.widgets.insert(name, v);
        Ok(wid)
    }
    pub fn div_sized(&self,gui:&mut Gui, w:i32, h:i32, vertical:bool, name:String)->Throwable<Widget>{
        let w = WidgetData{
            w_type:WidgetType::Div, vertical, x:0, y:0, w, h, h_padding:4, w_padding:4, text_height:12, children:Vec::new(), text:String::new(), name:name.clone(), on_update:None
        };
        let mut s = self.get();
        let v = Arc::new(Mutex::new(w));
        let wid = Widget{v:WeakNot::from(&v)};
        s.children.push(wid.clone());
        gui.widgets.insert(name, v);
        Ok(wid)
    }
    pub fn div(&self,gui:&mut Gui, vertical:bool, name:String)->Throwable<Widget>{
        self.div_sized(gui,0,0, vertical, name)
    }
    pub fn attach_text_input<T:Fn(&mut Gui, &mut WidgetData, GuiEvent)+'static>(&self, gui:&mut Gui,on_update:T, name:String)->Throwable<Widget>{
        let w = WidgetData{
            w_type:WidgetType::TextInput, vertical:true, x:0, y:0, w:0, h:0, h_padding:4, w_padding:4, text_height:12, children:Vec::new(), text:String::new(), name:name.clone(), on_update:Some(Arc::new(on_update))
        };
        let mut s = self.get();
        let v = Arc::new(Mutex::new(w));
        let wid = Widget{v:WeakNot::from(&v)};
        s.children.push(wid.clone());
        gui.widgets.insert(name, v);
        Ok(wid)
    }
    pub fn image(&self, gui:&mut Gui, image_name:String, w:i32, h:i32,name:String)->Throwable<Widget>{
        let w = WidgetData{
            w_type:WidgetType::Image, vertical:true, x:0, y:0, w, h, h_padding:4, w_padding:4, text_height:12, children:Vec::new(), text:image_name, name:name.clone(), on_update:None
        };
        let mut s = self.get();
        let v = Arc::new(Mutex::new(w));
        let wid = Widget{v:WeakNot::from(&v)};
        s.children.push(wid.clone());
        gui.widgets.insert(name, v);
        Ok(wid)
    }
    pub fn update_layout(&self, bounds:Bounds)->Throwable<()>{
        self.v.upgrade().unwrap().lock().unwrap().update_layout(bounds)
    }
     pub fn get_bounds_min(&self, max_bounds:Bounds)->Bounds{
        let ptr = self.get();
        ptr.get_bounds_min(max_bounds)
     }
     pub fn render(&self, image:&mut Image, images:&HashMap<String, Image>)->Throwable<()>{
        let t = self.v.upgrade().throw()?;
        let Ok(l) = t.lock() else{
            throw!("poison error");
        };
        l.render(image,images)?;
        Ok(())
     }
}
impl WidgetData{
    pub fn update_layout(&mut self,  bounds:Bounds)->Throwable<()>{
        let mins = self.get_bounds_min(bounds);
        if mins.w > bounds.w || mins.h> bounds.h{
            throw!(format!("error: created something bigger then its outer bounds"));
        }
        println!("{:#?}, {:#?}, {:#?}, {:#?}, {:#?}", self.x, self.y, self.w, self.h, bounds);
        self.w = if self.w >= bounds.w ||  self.w == 0{bounds.w}else{self.w};
        self.h =  if self.h >= bounds.h|| self.h == 0{bounds.h}else{self.h};
        self.x = bounds.x;
        self.y = bounds.y;       
        println!("updated:{:#?}, {:#?}, {:#?}, {:#?}", self.x, self.y, self.w, self.h);        
        if self.children.len() == 0{
            return Ok(());
        }
        let mut x = self.x+self.w_padding*2;
        let mut y = self.y + self.h_padding*2;
        let mut w = self.w_padding;
        let mut h = self.h_padding;
        for i in &self.children{
            let b = i.get_bounds_min(bounds);
            if self.vertical{
                h+= b.h+self.h_padding;
                if b.w+self.w_padding>w{
                    w = b.w;
                }
            }else{
                w += b.w+self.w_padding;
                if b.h+self.text_height>h{
                    h = b.h+self.h_padding;
                }
            }
        }
        let dx = if self.vertical{self.w - 2*self.w_padding} else{(self.w-w)/(self.children.len() as i32+1)};
        let dy = if !self.vertical{self.h - 2*self.h_padding} else{(self.h-h)/(self.children.len() as i32+1)};
        if self.vertical{
            y += dy/8;
        }else{
            x += dx/8;
        }
        for i in &self.children{
            let b0 = i.get_bounds_min(bounds);
            let b =  Bounds{x, y, w:b0.w + dx-self.w_padding*2, h:b0.h+dy-self.h_padding*2};
            i.update_layout(b)?;
            if self.vertical{
                y += b0.h+dy + self.h_padding;
            }else{
                x += b0.w+dx + self.w_padding;
            }    
        }
        Ok(())
    }
    pub fn get_bounds_min(&self, max_bounds:Bounds)->Bounds{
        let base = match self.w_type{
            WidgetType::None => Bounds { x: 0, y: 0, w: 0, h: 0 },
            WidgetType::Button => Bounds { x: 0, y: 0, w: 10, h: 10 },
            WidgetType::Text => {
                let p  = super::draw::Image::text_bounds_conservative(self.text_height, max_bounds.w, &self.text);

                Bounds{x:0, y:0, w:p.0, h:p.1}
            },
            WidgetType::Div => {
                Bounds{x:0, y:0, w:self.w, h:self.h}
            }
            WidgetType::TextInput => {
                Bounds{x:0, y:0, w:self.w, h:self.h}
            }
            WidgetType::Image => {
                 Bounds{x:0, y:0, w:self.w, h:self.h} 
            }
        };
        let mut bounds  = max_bounds;
        if self.w != 0 && self.w<bounds.w{
            bounds.w = self.w;
        }
        if self.h != 0 && self.h<bounds.h{
            bounds.h = self.h;
        }
        let mut bw = self.w_padding;
        let mut bh =self.h_padding;
        for i in &self.children{
            let b = i.get_bounds_min(bounds);
            if self.vertical{
                bh += b.h+self.h_padding;
                if bw< b.w{
                    bw = b.w;
                }
            }else{
                bw += b.w+self.w_padding;
                if bh< b.h{
                    bw = b.h
                }
            }
        }
        bw+= self.w_padding;
        bh += self.h_padding;
        if bw>base.w{
            bw = base.w;
        }
        if bh>base.h{
            bh = base.h;
        }
        Bounds { x: 0, y: 0, w: bw, h: bh }
    }
    pub fn render(&self, target:&mut Image, images:&HashMap<String, Image>)->Throwable<()>{
        match self.w_type{
            WidgetType::None => {

            }
            WidgetType::Button => {
              
                target.draw_rect_lines(self.x, self.y,self.w, self.h,1.0, BLACK);
            }
            WidgetType::Text => {
               // target.draw_rect_lines(self.x, self.y, self.w, self.h,1.0, BLACK);
                target.draw_text_box(self.x, self.y, self.w, self.h, &self.text, colors::BLACK);
            }
            WidgetType::Div => {
                target.draw_rect(self.x, self.y, self.w, self.h, WHITE);
                target.draw_rect_lines(self.x, self.y,self.w, self.h,1.0, BLACK);
            }
            WidgetType::TextInput => {
                target.draw_text_box_conservative(self.x, self.y, self.w, self.h, &self.text, colors::BLACK);
            }
            WidgetType::Image => {
                target.draw_rect_image(self.x, self.y, self.w, self.h, &images[&self.text]);
            }
        }
        for i in &self.children{
            i.render(target, images)?;
        }
        Ok(())
    }
}
impl Gui{
    pub fn new(x:i32, y:i32, w:i32, h:i32)->Self{
        let root = Arc::new(Mutex::new(WidgetData{ w_type:WidgetType::Div,
            vertical:false, x, y, w, h, children:Vec::new(), name:"root".to_string(),on_update:None , text:String::new(),text_height:12, w_padding:2, h_padding:2
        }));
        let mut tmp = Self { widgets: HashMap::new(), current_selected: None, input: GuiInput { events: Vec::new(), mouse_x: 0, mouse_y: 0, is_mouse_down: false }, destroy_queue: Vec::new(), root:root.clone()};
        tmp.widgets.insert("root".to_string(), root);
        tmp
    }
    pub fn get_widget(&self, name:&str)->Throwable<Widget>{
        Ok(Widget{v:WeakNot::from(&self.widgets[name])})
    }
    pub fn update(&mut self)->Throwable<()>{ 
        self.update_layout()?;
        for i in &self.widgets.clone(){
            let Ok(mut i0) = i.1.lock()else {
                continue;   
            };
            let inputs = GuiEvent::Nothing;
            if let Some(to_run) = i0.on_update.clone(){
                (*to_run)(self, &mut i0, inputs);
            }
        }
        Ok(())
    }
    pub fn update_layout(&mut self)->Throwable<()>{
        let mut r = self.root.lock().unwrap();
        let b = Bounds{x:r.x, y:r.y, w:r.w, h:r.h};
        r.update_layout(b)?;
        Ok(())
    }
    pub fn render(&mut self, target:&mut Image, images:&HashMap<String, Image>)->Throwable<()>{
        self.root.lock().unwrap().render(target, images)?;
        Ok(())
    }
    pub fn get_root(&self)->Widget{
        Widget { v:WeakNot::from(&self.root) }
    }
}