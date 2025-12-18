use crate::console::CmdParser;
use crate::executor::{Executor, ExitCode};
use crate::sys::SimpleOs;
use crate::{println, sys};
use alloc::rc::Rc;
use alloc::string::ToString;
use alloc::{boxed::Box, string::String, vec::Vec};
use async_trait::async_trait;
use core::cell::RefCell;

#[allow(unused)]
pub struct BuiltinCmds;

#[allow(unused)]
impl BuiltinCmds {
    pub fn new() -> Self {
        BuiltinCmds
    }

    pub fn cmd_reset(&self, _args: &Vec<String>) -> ExitCode {
        sys::SimpleOs::cpu().cpu_reset();
        0
    }

    pub fn cmd_ps(&self, _args: &Vec<String>) -> ExitCode {
        let task_list = Executor::task_list();
        println!("id\ttask");
        for (task_id, name) in task_list.iter() {
            println!("{}\t{}", task_id, name);
        }
        0
    }

    pub fn cmd_kill(&self, args: &Vec<String>) -> ExitCode {
        if let Some(id_str) = args.get(1) {
            if let Ok(id) = id_str.parse::<u16>() {
                Executor::kill(id);
                println!("Killed task with ID {}", id);
                0
            } else {
                println!("Invalid task ID: {}", id_str);
                1
            }
        } else {
            println!("Usage: kill <task_id>");
            2
        }
    }

    pub fn cmd_free(&self, _args: &Vec<String>) -> ExitCode {
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
        0
    }

    pub fn cmd_panic(&self, _args: &Vec<String>) -> ExitCode {
        SimpleOs::cpu().cpu_panic("MANUAL PANIC".to_string());
    }

    pub async fn cmd_pref(&self, _args: &Vec<String>) -> ExitCode {
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
        sys::join(f1, f2).await;
        0
    }
}

#[async_trait(?Send)]
impl CmdParser for BuiltinCmds {
    fn help(&self) -> &'static [(&'static str, &'static str)] {
        &[
            ("help|?", "Show this help message"),
            ("reset", "Perform a system reset"),
            ("ps", "Show running tasks"),
            ("kill", "Terminate a task"),
            ("free", "Show free memory"),
            ("pref", "Show task polling frequency"),
            ("panic", "Trigger a panic"),
        ]
    }

    async fn parse(&self, args: &Vec<String>) -> ExitCode {
        if let Some(cmd) = args.get(0) {
            match cmd.as_str() {
                "reset" => self.cmd_reset(&args),
                "ps" => self.cmd_ps(&args),
                "kill" => self.cmd_kill(&args),
                "free" => self.cmd_free(&args),
                "pref" => self.cmd_pref(&args).await,
                "panic" => self.cmd_panic(&args),
                _ => return 127, // Command not found
            }
        } else {
            127 // No command provided
        }
    }
}
