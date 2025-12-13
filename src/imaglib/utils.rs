use std::error::Error;

pub type Throwable<T> = Result<T, Box<dyn Error>>;
pub trait Throws<T:Sized> where Self:Sized{
    fn throw(self)->Throwable<T>{
        todo!()
    }
}
impl<T> Throws<T> for Option<T>{
    fn throw(self)->Throwable<T> {
        match self{
            Some(v)=> Ok(v),
            None=>Err(format!("Error: Option::<{}> contained None", std::any::type_name::<T>()).into())
        }
    }
}
impl<T,E:Error> Throws<T> for Result<T, E>  where Self:Into<Result<T, Box<dyn Error>>>{

    fn throw(self)->Throwable<T> {
        self.into()
    }
}
#[macro_export]
macro_rules! try_catch {
    ($to_try:block  catch ($err:ident) $caught:block) => {
        if let Err($err) = ((|| 
                    {
                        $to_try 
                        #[allow(unreachable_code)]
                        return Ok::<(), Box<dyn std::error::Error>>(());
                    }
                )()
            )
            $caught
    };
    ($to_try:block catch (_) $caught:block) => {
        if let Err(_) = ((|| {$to_try Ok::<(), Box<dyn std::error::Error>>(())})()) $caught
    };
}
#[macro_export]
macro_rules! throw {
    ($v:expr) => {
        {return Err($v.into())}
    };
}