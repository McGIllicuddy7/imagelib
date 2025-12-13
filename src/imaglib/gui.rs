use std::{collections::HashMap, sync::Arc};
pub struct WidgetData{
    pub x:i32, 
    pub y:i32, 
    pub w:i32, 
    pub h:i32,
}
pub struct Gui{
    pub widgets:Vec<Arc<WidgetData>>,
    pub prev_widgets:HashMap<String, Arc<WidgetData>>
}