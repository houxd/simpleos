use alloc::vec::Vec;
use alloc::string::String;

pub trait CmdParse {
    // fn cmd_parse(args: Vec<String>) -> Option<Vec<String>>;
    fn cmd_parse(args: Vec<String>) -> impl core::future::Future<Output = Option<Vec<String>>>;
}