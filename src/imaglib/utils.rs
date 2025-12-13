use std::{backtrace::Backtrace, error::Error, fmt::Display};

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
#[derive(Debug)]
pub struct Exception{
    pub error:Box<dyn Error>,
    pub trace:Backtrace,
}
impl Display for Exception{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl Error for Exception{}
#[macro_export]
macro_rules! throw {
    ($v:expr) => {
        {   
            let trace = std::backtrace::Backtrace::capture();
            return Err(crate::imaglib::utils::Exception{error:$v.into(), trace
            }.into());
        }
    };
}