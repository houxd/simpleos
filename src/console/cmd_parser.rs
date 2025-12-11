use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait CmdParser {
    fn help(&self) -> &'static [(&'static str, &'static str)];
    async fn parse(&self, args: Vec<String>) -> Option<Vec<String>>;
}
