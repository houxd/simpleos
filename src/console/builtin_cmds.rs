use core::cell::RefCell;

use crate::console::Cmds;
use crate::executor::Executor;
use crate::{println, sys};
use alloc::rc::Rc;
use alloc::{boxed::Box, string::String, vec::Vec};
use async_trait::async_trait;

#[allow(unused)]
pub struct BuiltinCmds;

#[allow(unused)]
impl BuiltinCmds {
    pub fn new() -> Self {
        BuiltinCmds
    }

    pub fn cmd_reset(&self, _args: &Vec<String>) {
        sys::SimpleOs::cpu().cpu_reset();
    }

    pub fn cmd_ps(&self, _args: &Vec<String>) {
        let task_list = Executor::task_list();
        println!("id\ttask");
        for (task_id, name) in task_list.iter() {
            println!("{}\t{}", task_id, name);
        }
    }

    pub fn cmd_kill(&self, args: &Vec<String>) {
        if let Some(id_str) = args.get(1) {
            if let Ok(id) = id_str.parse::<u16>() {
                Executor::kill(id);
                println!("Killed task with ID {}", id);
            } else {
                println!("Invalid task ID: {}", id_str);
            }
        } else {
            println!("Usage: kill <task_id>");
        }
    }

    pub fn cmd_free(&self, _args: &Vec<String>) {
        const MAX_BLOCK_INDEX: usize = 24; // 32;
        let mut total = 0u32;
        let mut block = 1u32 << MAX_BLOCK_INDEX; // 从4KB开始尝试分配
        let mut ptrs: [(*mut u8, usize); MAX_BLOCK_INDEX] =
            [(core::ptr::null_mut(), 0); MAX_BLOCK_INDEX];
        let mut ptrs_count = 0usize;

        while block > 0 && ptrs_count < MAX_BLOCK_INDEX {
            if let Ok(layout) = core::alloc::Layout::from_size_align(block as usize, 4) {
                let ptr = unsafe { alloc::alloc::alloc(layout) };
                if !ptr.is_null() {
                    total += block;
                    ptrs[ptrs_count] = (ptr, block as usize);
                    ptrs_count += 1;
                }
            }
            block /= 2; // 减小块大小
        }

        // 释放所有分配的内存，使用正确的大小
        for i in 0..ptrs_count {
            let (ptr, size) = ptrs[i];
            if !ptr.is_null() {
                if let Ok(layout) = core::alloc::Layout::from_size_align(size, 4) {
                    unsafe { alloc::alloc::dealloc(ptr, layout) };
                }
            }
        }

        println!("Free memory: {} bytes", total);
    }

    pub fn cmd_panic(&self, _args: &Vec<String>) {
        panic!("MANUAL PANIC");
    }

    pub fn cmd_pref(&self, _args: &Vec<String>) {
        let c = Rc::new(RefCell::new(0u32));
        let f1 = async {
            loop {
                if let Ok(mut cc) = c.try_borrow_mut() {
                    *cc += 1;
                }
                sys::yield_now().await;
            }
        };
        let f2 = async {
            loop {
                sys::sleep_ms(1000).await;
                if let Ok(mut cc) = c.try_borrow_mut() {
                    println!("poll freq: {} times/sec", *cc);
                    *cc = 0;
                }
            }
        };
        sys::join(f1, f2);
    }
}

#[async_trait(?Send)]
impl Cmds for BuiltinCmds {
    fn help(&self) -> &'static [(&'static str, &'static str)] {
        &[
            ("help|?", "Show this help message"),
            ("reset", "Perform a system reset"),
            // ("date", "Get or set the system date and time"),
            ("ps", "Show running tasks"),
            ("kill", "Terminate a task"),
            ("free", "Show free memory"),
            ("pref", "Show task polling frequency"),
            ("panic", "Trigger a panic"),
        ]
    }

    async fn parse(&self, args: Vec<String>) -> Option<Vec<String>> {
        if let Some(cmd) = args.get(0) {
            match cmd.as_str() {
                "reset" => self.cmd_reset(&args),
                "ps" => self.cmd_ps(&args),
                "kill" => self.cmd_kill(&args),
                "free" => self.cmd_free(&args),
                "pref" => self.cmd_pref(&args),
                "panic" => self.cmd_panic(&args),
                _ => return Some(args),
            }
            return None;
        } else {
            return Some(args);
        }
    }
}
