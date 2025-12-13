


pub mod imaglib;


pub fn main(){
    try_catch!(
        {
            throw!("testing 123".to_string());
        } catch (err) {
            println!("{:#?}",err);
        }
    );
    println!("done");
}

