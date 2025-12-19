

use std::{collections::HashMap, error::Error};

use crate::imaglib::{draw::Image, gui::Gui};




pub mod imaglib;


pub fn main()->Result<(), Box<dyn Error>>{
    let mut g = Gui::new(0, 0, 1000, 1000);
    let root = g.get_root();
    let d3 = root.div(&mut g,  true, "d3".into())?;
    d3.text(&mut g, "hi toast I love you".into(), "toast".into())?;
    let div = root.div(&mut g, true, "div".to_string()).unwrap();
    div.text(&mut g,"testing 1 2 3".to_string(), "text".to_string()).unwrap();
    div.text(&mut g,"testing 4 5 6".to_string(), "text".to_string()).unwrap();
    let d2 = root.div(&mut g, true, "div2".to_string())?;
    d2.text(&mut g, "CATS :3".to_string(), "cats".to_string())?;
    d2.text(&mut g, "more cats".to_string(), "more cats".to_string())?;
    d2.image(&mut g,"florence".into(), 300, 300, "ethel_cain.png".into())?;
    let mut img = Image::new(800, 800);
    g.update().unwrap();
    let mut map = HashMap::new();
    map.insert("florence".into(), Image::load("image.png")?);
    g.render(&mut img, &map).unwrap();
    img.draw_forever();
    Ok(())
}   

