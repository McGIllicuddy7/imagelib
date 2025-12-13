


pub mod imaglib;


pub fn main(){
    try_catch!(
        {
            throw!(format!("{:#?}",42));
        } catch (err) {
            println!("{:#?}",err);
        }
    );
    println!("done");
}

