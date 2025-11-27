use crate::{print, println, singleton};
use alloc::{boxed::Box, format, string::String, string::ToString, vec::Vec};
use anyhow::{anyhow, Result};
use core::any::Any;

pub trait FsHandle: Any {
    #[allow(unused)]
    fn as_any(&self) -> &dyn Any;
    #[allow(unused)]
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn info(&mut self) -> Result<Box<dyn FsInfo>>;
    fn mount(&mut self) -> Result<()>;
    fn unmount(&mut self) -> Result<()>;
    fn format(&mut self) -> Result<()>;
    fn mkdir(&mut self, path: &str) -> Result<()>;
    fn unlink(&mut self, path: &str) -> Result<()>;
    fn rename(&mut self, old_path: &str, new_path: &str) -> Result<()>;
    fn stat(&mut self, path: &str) -> Result<Box<dyn DirEntry>>;
    fn readdir(&mut self, path: &str) -> Result<Vec<Box<dyn DirEntry>>>;
    fn sync(&mut self) -> Result<()>;
    fn open(&mut self, path: &str, mode: &str) -> Result<Box<dyn FileHandle>>;
    fn close(&mut self, file_node: &mut Box<dyn FileHandle>) -> Result<()>;
    fn flush(&mut self, file_node: &mut Box<dyn FileHandle>) -> Result<()>;
    fn read(&mut self, file_node: &mut Box<dyn FileHandle>, buf: &mut [u8]) -> Result<usize>;
    fn write(&mut self, file_node: &mut Box<dyn FileHandle>, buf: &[u8]) -> Result<usize>;
    #[allow(unused)]
    fn seek(
        &mut self,
        file_node: &mut Box<dyn FileHandle>,
        pos: isize,
        whence: Whence,
    ) -> Result<isize>;
}

pub trait FileHandle: Any {
    #[allow(unused)]
    fn as_any(&self) -> &dyn Any;
    #[allow(unused)]
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait FsInfo: Any {
    #[allow(unused)]
    fn as_any(&self) -> &dyn Any;
    #[allow(unused)]
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn total(&self) -> isize;
    fn used(&self) -> isize;
    fn free(&self) -> isize;
}

#[allow(unused)]
#[allow(non_camel_case_types)]
pub enum Whence {
    SEEK_SET = 0,
    SEEK_CUR = 1,
    SEEK_END = 2,
}

pub struct File {
    node: Box<dyn FileHandle>,
    fs_index: usize,
    closed: bool,
}

impl File {
    fn open(path: &str, mode: &str) -> Result<File> {
        let (fs, fs_index, path) = Fs::path_to_fs_index(path)?;
        let node = fs.open(path, mode)?;
        Ok(File {
            node,
            fs_index,
            closed: false,
        })
    }
    fn close(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }
        let fs = &mut Fs::ref_mut().fstab[self.fs_index].fs;
        fs.close(&mut self.node)?;
        self.closed = true;
        Ok(())
    }
    fn flush(&mut self) -> Result<()> {
        if self.closed {
            return Err(anyhow!("File is already closed"));
        }
        let fs = &mut Fs::ref_mut().fstab[self.fs_index].fs;
        fs.flush(&mut self.node)
    }
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.closed {
            return Err(anyhow!("File is already closed"));
        }
        let fs = &mut Fs::ref_mut().fstab[self.fs_index].fs;
        fs.read(&mut self.node, buf)
    }
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if self.closed {
            return Err(anyhow!("File is already closed"));
        }
        let fs = &mut Fs::ref_mut().fstab[self.fs_index].fs;
        fs.write(&mut self.node, buf)
    }
    #[allow(unused)]
    fn seek(&mut self, pos: isize, whence: Whence) -> Result<isize> {
        if self.closed {
            return Err(anyhow!("File is already closed"));
        }
        let fs = &mut Fs::ref_mut().fstab[self.fs_index].fs;
        fs.seek(&mut self.node, pos, whence)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

pub trait DirEntry: Any {
    #[allow(unused)]
    fn as_any(&self) -> &dyn Any;
    #[allow(unused)]
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn name(&self) -> &str;
    fn is_file(&self) -> bool;
    fn is_dir(&self) -> bool;
    fn size(&self) -> usize;
}

pub struct Fs {
    fstab: &'static mut [FsEntry],
    cwd: String,
}

pub struct FsEntry {
    pub mount_point: &'static str,
    pub fs: &'static mut dyn FsHandle,
}

singleton!(Fs {
    fstab: &mut [],
    cwd: String::from("/"),
});

impl Fs {
    pub fn init(fstab: &'static mut [FsEntry]) -> Result<()> {
        Fs::ref_mut().fstab = fstab;

        for entry in Fs::ref_mut().fstab.iter_mut() {
            if entry.fs.mount().is_err() {
                println!(
                    "Mounting {} failed, try mount after formatting...",
                    entry.mount_point
                );
                entry.fs.format()?;
                entry.fs.mount()?;
            }
            println!("Mounted {} successfully", entry.mount_point);
        }

        Ok(())
    }
    fn path_to_fs(path: &str) -> Result<(&'static mut dyn FsHandle, &str)> {
        if path == "/" {
            return Err(anyhow!("Root path does not belong to any filesystem"));
        }
        for entry in Fs::ref_mut().fstab.iter_mut() {
            if path.starts_with(entry.mount_point) {
                let path = &path[entry.mount_point.len()..];
                let path = if path.is_empty() { "/" } else { path };
                return Ok((entry.fs, path));
            }
        }
        Err(anyhow!("Filesystem not found for path: {}", path))
    }
    fn path_to_fs_index(path: &str) -> Result<(&'static mut dyn FsHandle, usize, &str)> {
        if path == "/" {
            return Err(anyhow!("Root path does not belong to any filesystem"));
        }
        for (index, entry) in Fs::ref_mut().fstab.iter_mut().enumerate() {
            if path.starts_with(entry.mount_point) {
                let path = &path[entry.mount_point.len()..];
                let path = if path.is_empty() { "/" } else { path };
                return Ok((entry.fs, index, path));
            }
        }
        Err(anyhow!("Filesystem not found for path: {}", path))
    }
    fn mount_point_to_fs(mount_point: &str) -> Result<&'static mut dyn FsHandle> {
        for entry in Fs::ref_mut().fstab.iter_mut() {
            if entry.mount_point == mount_point {
                return Ok(entry.fs);
            }
        }
        Err(anyhow!("Mount point not found: {}", mount_point))
    }
    pub fn info(mount_point: &str) -> Result<Box<dyn FsInfo>> {
        Fs::mount_point_to_fs(mount_point)?.info()
    }
    pub fn mount(mount_point: &str) -> Result<()> {
        Fs::mount_point_to_fs(mount_point)?.mount()
    }
    pub fn unmount(mount_point: &str) -> Result<()> {
        Fs::mount_point_to_fs(mount_point)?.unmount()
    }
    pub fn format(mount_point: &str) -> Result<()> {
        Fs::mount_point_to_fs(mount_point)?.format()
    }
    pub fn mkdir(path: &str) -> Result<()> {
        if path == "/" {
            return Err(anyhow!("Cannot create root directory"));
        }
        let (fs, path) = Fs::path_to_fs(path)?;
        fs.mkdir(path)
    }
    pub fn unlink(path: &str) -> Result<()> {
        if path == "/" {
            return Err(anyhow!("Cannot delete root directory"));
        }
        let (fs, path) = Fs::path_to_fs(path)?;
        fs.unlink(path)
    }
    pub fn rename(old_path: &str, new_path: &str) -> Result<()> {
        if old_path == "/" || new_path == "/" {
            return Err(anyhow!("Cannot rename root directory"));
        }
        let (old_fs, old_fs_idx, old_path) = Fs::path_to_fs_index(old_path)?;
        let (new_fs, new_fs_idx, new_path) = Fs::path_to_fs_index(new_path)?;
        if new_fs_idx == old_fs_idx {
            return old_fs.rename(old_path, new_path);
        } else {
            let _ = new_fs;
            return Err(anyhow!("Cannot rename across different filesystems"));
        }
    }
    pub fn stat(path: &str) -> Result<Box<dyn DirEntry>> {
        if path == "/" {
            struct RootDirEntry;
            impl DirEntry for RootDirEntry {
                fn as_any(&self) -> &dyn Any {
                    self
                }
                fn as_any_mut(&mut self) -> &mut dyn Any {
                    self
                }
                fn name(&self) -> &str {
                    "/"
                }
                fn is_file(&self) -> bool {
                    false
                }
                fn is_dir(&self) -> bool {
                    true
                }
                fn size(&self) -> usize {
                    0
                }
            }
            return Ok(Box::new(RootDirEntry));
        }
        let (fs, path) = Fs::path_to_fs(path)?;
        fs.stat(path)
    }
    pub fn readdir(path: &str) -> Result<Vec<Box<dyn DirEntry>>> {
        if path == "/" {
            let mut entries: Vec<Box<dyn DirEntry>> = Vec::new();
            for entry in Fs::ref_mut().fstab.iter() {
                struct MountPointEntry {
                    name: String,
                }
                impl DirEntry for MountPointEntry {
                    fn as_any(&self) -> &dyn Any {
                        self
                    }
                    fn as_any_mut(&mut self) -> &mut dyn Any {
                        self
                    }
                    fn name(&self) -> &str {
                        &self.name
                    }
                    fn is_file(&self) -> bool {
                        false
                    }
                    fn is_dir(&self) -> bool {
                        true
                    }
                    fn size(&self) -> usize {
                        0
                    }
                }
                entries.push(Box::new(MountPointEntry {
                    name: entry.mount_point.trim_start_matches('/').to_string(),
                }));
            }
            return Ok(entries);
        }
        let (fs, path) = Fs::path_to_fs(path)?;
        fs.readdir(path)
    }
    pub fn sync() -> Result<()> {
        for entry in Fs::ref_mut().fstab.iter_mut() {
            entry.fs.sync()?;
        }
        Ok(())
    }
    #[allow(unused)]
    pub fn exists(path: &str) -> bool {
        Fs::stat(path).is_ok()
    }
    pub fn change_dir(path: &str) -> Result<()> {
        let stat = Fs::stat(path)?;
        if !stat.is_dir() {
            return Err(anyhow!("{} is not a directory", path));
        }
        Fs::ref_mut().cwd = path.to_string();
        Ok(())
    }
    pub fn get_cwd() -> String {
        Fs::ref_mut().cwd.clone()
    }
    pub fn to_absolute_path(path: &str) -> String {
        // println!("1> {}", path);
        let path = if path.starts_with('/') {
            path.to_string()
        } else {
            let cwd = Fs::ref_mut().cwd.clone();
            if cwd.ends_with('/') {
                format!("{}{}", cwd, path)
            } else {
                format!("{}/{}", cwd, path)
            }
        };
        // println!("2> {}", path);

        // Normalize path (remove redundant slashes)
        let parts: Vec<&str> = path.split('/').collect();
        let mut normalized_parts: Vec<&str> = Vec::new();
        for part in parts {
            if part == "." || part.is_empty() {
                continue;
            } else if part == ".." {
                normalized_parts.pop();
            } else {
                normalized_parts.push(part);
            }
        }
        let path = normalized_parts.join("/");
        // println!("3> {}", path);

        // Ensure leading slash
        if !path.starts_with('/') {
            format!("/{}", path)
        } else {
            path
        }
    }
    pub async fn on_cmd(args: Vec<String>) -> Option<Vec<String>> {
        if let Some(cmd) = args.get(0) {
            match cmd.as_str() {
                "help" | "?" => {
                    println!("Filesystem commands:");
                    println!("  {:<40} {}", "df", "Show filesystem disk usage");
                    println!(
                        "  {:<40} {}",
                        "mount <mount_point>", "Mount the filesystem at mount_point"
                    );
                    println!(
                        "  {:<40} {}",
                        "unmount <mount_point>", "Unmount the filesystem at mount_point"
                    );
                    println!(
                        "  {:<40} {}",
                        "format <mount_point>", "Format the filesystem at mount_point"
                    );
                    println!(
                        "  {:<40} {}",
                        "info <mount_point>",
                        "Show information about the filesystem at mount_point"
                    );
                    println!("  {:<40} {}", "mkdir <path>", "Create a directory at path");
                    println!(
                        "  {:<40} {}",
                        "rm <path>", "Delete the file or directory at path"
                    );
                    println!(
                        "  {:<40} {}",
                        "mv <old_path> <new_path>", "Rename a file or directory"
                    );
                    println!("  {:<40} {}", "sync", "Synchronize all filesystems");
                    println!(
                        "  {:<40} {}",
                        "ls <path>", "List directory contents at path"
                    );
                    println!(
                        "  {:<40} {}",
                        "cd <path>", "Change current directory to path"
                    );
                    println!("  {:<40} {}", "pwd", "Print current working directory");
                    println!(
                        "  {:<40} {}",
                        "touch <path>", "Create an empty file at path"
                    );
                    println!(
                        "  {:<40} {}",
                        "cat <path>", "Display the contents of the file at path"
                    );
                    println!(
                        "  {:<40} {}",
                        "write <path> <content>", "Write content to the file at path"
                    );
                    return Some(args);
                }
                "df" => {
                    println!("Mount Point    Total       Used        Free");
                    for entry in Fs::ref_mut().fstab.iter_mut() {
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
                    return None;
                }
                "mount" => {
                    if let Some(mount_point) = args.get(1) {
                        match Fs::mount(mount_point) {
                            Ok(_) => println!("Mounted {}", mount_point),
                            Err(e) => println!("Error mounting {}: {}", mount_point, e),
                        }
                    } else {
                        println!("Usage: mount <mount_point>");
                    }
                    return None;
                }
                "unmount" => {
                    if let Some(mount_point) = args.get(1) {
                        match Fs::unmount(mount_point) {
                            Ok(_) => println!("Unmounted {}", mount_point),
                            Err(e) => println!("Error unmounting {}: {}", mount_point, e),
                        }
                    } else {
                        println!("Usage: unmount <mount_point>");
                    }
                    return None;
                }
                "format" => {
                    if let Some(mount_point) = args.get(1) {
                        match Fs::format(mount_point) {
                            Ok(_) => println!("Formatted {}", mount_point),
                            Err(e) => println!("Error formatting {}: {}", mount_point, e),
                        }
                    } else {
                        println!("Usage: format <mount_point>");
                    }
                    return None;
                }
                "info" => {
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
                            }
                            Err(e) => println!("Error getting info for {}: {}", mount_point, e),
                        }
                    } else {
                        println!("Usage: info <mount_point>");
                    }
                    return None;
                }
                "mkdir" => {
                    if let Some(path) = args.get(1) {
                        let path = Fs::to_absolute_path(path);
                        match Fs::mkdir(&path) {
                            Ok(_) => {}
                            Err(e) => println!("Error creating directory {}: {}", path, e),
                        }
                    } else {
                        println!("Usage: mkdir <path>");
                    }
                    return None;
                }
                "rm" => {
                    if let Some(path) = args.get(1) {
                        let path = Fs::to_absolute_path(path);
                        match Fs::unlink(&path) {
                            Ok(_) => {}
                            Err(e) => println!("Error deleting {}: {}", path, e),
                        }
                    } else {
                        println!("Usage: rm <path>");
                    }
                    return None;
                }
                "mv" => {
                    if let (Some(old_path), Some(new_path)) = (args.get(1), args.get(2)) {
                        let old_path = Fs::to_absolute_path(old_path);
                        let new_path = Fs::to_absolute_path(new_path);
                        match Fs::rename(&old_path, &new_path) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Error renaming {} to {}: {}", old_path, new_path, e)
                            }
                        }
                    } else {
                        println!("Usage: mv <old_path> <new_path>");
                    }
                    return None;
                }
                "sync" => {
                    match Fs::sync() {
                        Ok(_) => println!("Filesystem synchronized"),
                        Err(e) => println!("Error synchronizing filesystem: {}", e),
                    }
                    return None;
                }
                "ls" => {
                    let path = if let Some(p) = args.get(1) {
                        p.as_str()
                    } else {
                        Fs::ref_mut().cwd.as_str()
                    };
                    match Fs::readdir(path) {
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
                        }
                        Err(e) => println!("Error reading directory {}: {}", path, e),
                    }
                    return None;
                }
                "cd" => {
                    if let Some(path) = args.get(1) {
                        let path = Fs::to_absolute_path(path);
                        match Fs::change_dir(&path) {
                            Ok(_) => {}
                            Err(e) => println!("Error changing directory to {}: {}", path, e),
                        }
                    } else {
                        println!("Usage: cd <path>");
                    }
                    return None;
                }
                "pwd" => {
                    let cwd = Fs::get_cwd();
                    println!("{}", cwd);
                    return None;
                }
                "touch" => {
                    if let Some(path) = args.get(1) {
                        let path = Fs::to_absolute_path(path);
                        match File::open(&path, "w") {
                            Ok(mut file) => match file.close() {
                                Ok(_) => {}
                                Err(e) => println!("Error closing file {}: {}", path, e),
                            },
                            Err(e) => println!("Error creating file {}: {}", path, e),
                        }
                    } else {
                        println!("Usage: touch <path>");
                    }
                    return None;
                }
                "cat" => {
                    if let Some(path) = args.get(1) {
                        let path = Fs::to_absolute_path(path);
                        match File::open(&path, "r") {
                            Ok(mut file) => {
                                let mut buffer = [0u8; 256];
                                loop {
                                    match file.read(&mut buffer) {
                                        Ok(0) => break,
                                        Ok(n) => {
                                            let content = core::str::from_utf8(&buffer[..n])
                                                .unwrap_or("[Invalid UTF-8]");
                                            print!("{}", content);
                                        }
                                        Err(e) => {
                                            println!("Error reading file {}: {}", path, e);
                                            break;
                                        }
                                    }
                                }
                                match file.close() {
                                    Ok(_) => {}
                                    Err(e) => println!("Error closing file {}: {}", path, e),
                                }
                            }
                            Err(e) => println!("Error opening file {}: {}", path, e),
                        }
                    } else {
                        println!("Usage: cat <path>");
                    }
                    return None;
                }
                "write" => {
                    if let Some(path) = args.get(1) {
                        let path = Fs::to_absolute_path(path);
                        match File::open(&path, "w") {
                            Ok(mut file) => {
                                if let Some(content) = args.get(2) {
                                    let content = content.as_bytes();
                                    match file.write(content) {
                                        Ok(_) => match file.flush() {
                                            Ok(_) => {}
                                            Err(e) => {
                                                println!("Error flushing file {}: {}", path, e)
                                            }
                                        },
                                        Err(e) => println!("Error writing to file {}: {}", path, e),
                                    }
                                } else {
                                    println!("Usage: write <path> <content>");
                                }
                                match file.close() {
                                    Ok(_) => {}
                                    Err(e) => println!("Error closing file {}: {}", path, e),
                                }
                            }
                            Err(e) => println!("Error opening file {}: {}", path, e),
                        }
                    } else {
                        println!("Usage: write <path> <content>");
                    }
                    return None;
                }
                _ => {}
            }
        }
        Some(args)
    }
}
