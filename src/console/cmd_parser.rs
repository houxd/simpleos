use core::future::Future;
use core::pin::Pin;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait CmdParser {
    fn help(&self) -> &'static [(&'static str, &'static str)];
    async fn parse(&self, args: Vec<String>) -> Option<Vec<String>>;
}

// #[async_trait(?Send)]
// pub trait Cmd {
//     fn name(&self) -> &'static str;
//     fn help(&self) -> &'static str;
//     async fn exec(&self, args: &Vec<String>);
// }
