use alloc::boxed::Box;
use alloc::string::String;
use core::future::Future;
use core::pin::Pin;
use crate::executor::ExitCode;

pub type Arguments<'a> = &'a [String];

pub struct Runnable {
    name: String,
    func: Box<dyn Fn(Arguments) -> Pin<Box<dyn Future<Output = ExitCode>>>>,
}

impl Runnable {
    pub fn new<F, Fut>(name: String, async_func: F) -> Self
    where
        F: Fn(Arguments) -> Fut + 'static,
        Fut: Future<Output = ExitCode> + 'static,
    {
        Runnable {
            name,
            func: Box::new(move |args| Box::pin(async_func(args))),
        }
    }

    #[inline]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    #[inline]
    pub fn run(&self, args: Arguments) -> Pin<Box<dyn Future<Output = ExitCode>>> {
        (*self.func)(args)
    }
}
