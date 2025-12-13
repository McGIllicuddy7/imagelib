


pub mod imaglib;


pub fn main(){
    try_catch!(
        {
            throw!("testing 123");
        } catch (err) {
            println!("{:#?}",err);
        }
    );
    println!("done");
}

