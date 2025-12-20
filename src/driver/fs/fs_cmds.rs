use crate::console::CmdParser;
use crate::driver::fs::{File, Fs};
use crate::executor::{ ExitCode};
use crate::{print, println};
use alloc::{boxed::Box, string::String, vec::Vec};
use async_trait::async_trait;

#[allow(unused)]
pub struct FsCmds;

#[allow(unused)]
impl FsCmds {
    pub fn new() -> Self {
        FsCmds
    }
    fn cmd_df(&self, _args: &Vec<String>) -> ExitCode {
        println!("Mount Point    Total       Used        Free");
        for entry in Fs::fstab().iter_mut() {
            match entry.fs.info() {
                Ok(info) => {
                    println!(
                        "{:<14} {:<10} {:<10} {:<10}",
                        entry.mount_point,
                        info.total(),
                        info.used(),
                        info.free()
                    );
                }
                Err(e) => {
                    println!("Error getting info for {}: {}", entry.mount_point, e)
                }
            }
        }
        0
    }
    fn cmd_mount(&self, args: &Vec<String>) -> ExitCode {
        if let Some(mount_point) = args.get(1) {
            match Fs::mount(mount_point) {
                Ok(_) => {
                    println!("Mounted {}", mount_point);
                    0
                }
                Err(e) => {
                    println!("Error mounting {}: {}", mount_point, e);
                    1
                }
            }
        } else {
            println!("Usage: mount <mount_point>");
            2
        }
    }
    fn cmd_unmount(&self, args: &Vec<String>) -> ExitCode {
        if let Some(mount_point) = args.get(1) {
            match Fs::unmount(mount_point) {
                Ok(_) => {
                    println!("Unmounted {}", mount_point);
                    0
                }
                Err(e) => {
                    println!("Error unmounting {}: {}", mount_point, e);
                    1
                }
            }
        } else {
            println!("Usage: unmount <mount_point>");
            2
        }
    }
    fn cmd_format(&self, args: &Vec<String>) -> ExitCode {
        if let Some(mount_point) = args.get(1) {
            match Fs::format(mount_point) {
                Ok(_) => {
                    println!("Formatted {}", mount_point);
                    0
                }
                Err(e) => {
                    println!("Error formatting {}: {}", mount_point, e);
                    1
                }
            }
        } else {
            println!("Usage: format <mount_point>");
            2
        }
    }
    fn cmd_info(&self, args: &Vec<String>) -> ExitCode {
        if let Some(mount_point) = args.get(1) {
            match Fs::info(mount_point) {
                Ok(info) => {
                    println!(
                        "FS Info for {}: Total: {}, Used: {}, Free: {}",
                        mount_point,
                        info.total(),
                        info.used(),
                        info.free()
                    );
                    0
                }
                Err(e) => {
                    println!("Error getting info for {}: {}", mount_point, e);
                    1
                }
            }
        } else {
            println!("Usage: info <mount_point>");
            2
        }
    }
    fn cmd_mkdir(&self, args: &Vec<String>) -> ExitCode {
        if let Some(path) = args.get(1) {
            let path = Fs::to_absolute_path(path);
            match Fs::mkdir(&path) {
                Ok(_) => 0,
                Err(e) => {
                    println!("Error creating directory {}: {}", path, e);
                    1
                }
            }
        } else {
            println!("Usage: mkdir <path>");
            2
        }
    }
    fn cmd_rm(&self, args: &Vec<String>) -> ExitCode {
        if let Some(path) = args.get(1) {
            let path = Fs::to_absolute_path(path);
            match Fs::unlink(&path) {
                Ok(_) => 0,
                Err(e) => {
                    println!("Error deleting {}: {}", path, e);
                    1
                }
            }
        } else {
            println!("Usage: rm <path>");
            2
        }
    }
    fn cmd_mv(&self, args: &Vec<String>) -> ExitCode {
        if let (Some(old_path), Some(new_path)) = (args.get(1), args.get(2)) {
            let old_path = Fs::to_absolute_path(old_path);
            let new_path = Fs::to_absolute_path(new_path);
            match Fs::rename(&old_path, &new_path) {
                Ok(_) => 0,
                Err(e) => {
                    println!("Error renaming {} to {}: {}", old_path, new_path, e);
                    1
                }
            }
        } else {
            println!("Usage: mv <old_path> <new_path>");
            2
        }
    }
    fn cmd_sync(&self, _args: &Vec<String>) -> ExitCode {
        match Fs::sync() {
            Ok(_) => {
                println!("Filesystem synchronized");
                0
            }
            Err(e) => {
                println!("Error synchronizing filesystem: {}", e);
                1
            }
        }
    }
    fn cmd_ls(&self, args: &Vec<String>) -> ExitCode {
        let path = if let Some(p) = args.get(1) {
            p.clone()
        } else {
            Fs::cwd()
        };
        match Fs::readdir(path.as_str()) {
            Ok(entries) => {
                for entry in &entries {
                    if entry.is_dir() {
                        println!("DIR\t-\t{}", entry.name());
                    }
                }
                for entry in &entries {
                    if entry.is_file() {
                        println!("FILE\t{}\t{}", entry.size(), entry.name());
                    }
                }
                0
            }
            Err(e) => {
                println!("Error reading directory {}: {}", path, e);
                1
            }
        }
    }
    fn cmd_cd(&self, args: &Vec<String>) -> ExitCode {
        if let Some(path) = args.get(1) {
            let path = Fs::to_absolute_path(path);
            match Fs::change_dir(&path) {
                Ok(_) => 0,
                Err(e) => {
                    println!("Error changing directory to {}: {}", path, e);
                    1
                }
            }
        } else {
            println!("Usage: cd <path>");
            2
        }
    }
    fn cmd_pwd(&self, _args: &Vec<String>) -> ExitCode {
        let cwd = Fs::get_cwd();
        println!("{}", cwd);
        0
    }
    fn cmd_touch(&self, args: &Vec<String>) -> ExitCode {
        if let Some(path) = args.get(1) {
            let path = Fs::to_absolute_path(path);
            match File::open(&path, "w") {
                Ok(mut file) => match file.close() {
                    Ok(_) => 0,
                    Err(e) => {
                        println!("Error closing file {}: {}", path, e);
                        1
                    }
                },
                Err(e) => {
                    println!("Error creating file {}: {}", path, e);
                    1
                }
            }
        } else {
            println!("Usage: touch <path>");
            2
        }
    }
    fn cmd_cat(&self, args: &Vec<String>) -> ExitCode {
        if let Some(path) = args.get(1) {
            let path = Fs::to_absolute_path(path);
            match crate::driver::fs::File::open(&path, "r") {
                Ok(mut file) => {
                    let mut buffer = [0u8; 256];
                    loop {
                        match file.read(&mut buffer) {
                            Ok(0) => break,
                            Ok(n) => {
                                let content =
                                    core::str::from_utf8(&buffer[..n]).unwrap_or("[Invalid UTF-8]");
                                print!("{}", content);
                            }
                            Err(e) => {
                                println!("Error reading file {}: {}", path, e);
                                break;
                            }
                        }
                    }
                    println!();
                    match file.close() {
                        Ok(_) => 0,
                        Err(e) => {
                            println!("Error closing file {}: {}", path, e);
                            1
                        }
                    }
                }
                Err(e) => {
                    println!("Error opening file {}: {}", path, e);
                    1
                }
            }
        } else {
            println!("Usage: cat <path>");
            2
        }
    }

    fn cmd_write(&self, args: &Vec<String>) -> ExitCode {
        if let Some(path) = args.get(1) {
            let path = Fs::to_absolute_path(path);
            match crate::driver::fs::File::open(&path, "w") {
                Ok(mut file) => {
                    if let Some(content) = args.get(2) {
                        let content = content.as_bytes();
                        match file.write(content) {
                            Ok(_) => match file.flush() {
                                Ok(_) => 0,
                                Err(e) => {
                                    println!("Error flushing file {}: {}", path, e);
                                    1
                                }
                            },
                            Err(e) => {
                                println!("Error writing to file {}: {}", path, e);
                                1
                            }
                        }
                    } else {
                        println!("Usage: write <path> <content>");
                        2
                    }
                }
                Err(e) => {
                    println!("Error opening file {}: {}", path, e);
                    1
                }
            }
        } else {
            println!("Usage: write <path> <content>");
            2
        }
    }
}

#[async_trait(?Send)]
impl CmdParser for FsCmds {
    #[rustfmt::skip]
    fn help(&self) -> &'static [(&'static str, &'static str)] {
        &[
            ("df", "Show filesystem disk usage"),
            ("mount <mount_point>", "Mount the filesystem at mount_point"),
            ("unmount <mount_point>", "Unmount the filesystem at mount_point"),
            ("format <mount_point>", "Format the filesystem at mount_point"),
            ("info <mount_point>", "Show information about the filesystem at mount_point"),
            ("mkdir <path>", "Create a directory at path"),
            ("rm <path>", "Delete the file or directory at path"),
            ("mv <old_path> <new_path>", "Rename a file or directory"),
            ("sync", "Synchronize all filesystems"),
            ("ls <path>", "List directory contents at path"),
            ("cd <path>", "Change current directory to path"),
            ("pwd", "Print current working directory"),
            ("touch <path>", "Create an empty file at path"),
            ("cat <path>", "Display the contents of the file at path"),
            ("write <path> <content>", "Write content to the file at path"),
        ]
    }

    async fn parse(&self, args: &Vec<String>) -> ExitCode {
        if let Some(cmd) = args.get(0) {
            match cmd.as_str() {
                "df" => self.cmd_df(args),
                "mount" => self.cmd_mount(args),
                "unmount" => self.cmd_unmount(args),
                "format" => self.cmd_format(args),
                "info" => self.cmd_info(args),
                "mkdir" => self.cmd_mkdir(args),
                "rm" => self.cmd_rm(args),
                "mv" => self.cmd_mv(args),
                "sync" => self.cmd_sync(args),
                "ls" => self.cmd_ls(args),
                "cd" => self.cmd_cd(args),
                "pwd" => self.cmd_pwd(args),
                "touch" => self.cmd_touch(args),
                "cat" => self.cmd_cat(args),
                "write" => self.cmd_write(args),
                _ => return 127, // Command not found
            }
        } else {
            127 // No command provided
        }
    }
}
