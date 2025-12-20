use crate::{driver::Driver, println, singleton};
use alloc::{boxed::Box, format, string::String, string::ToString, vec::Vec};
use anyhow::{anyhow, Result};
use core::any::Any;

pub trait FsHandle: Driver + Any {
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
    pub fn open(path: &str, mode: &str) -> Result<File> {
        let (fs, fs_index, path) = Fs::path_to_fs_index(path)?;
        let node = fs.open(path, mode)?;
        Ok(File {
            node,
            fs_index,
            closed: false,
        })
    }
    pub fn close(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }
        let fs = &mut Fs::get_mut().fstab[self.fs_index].fs;
        fs.close(&mut self.node)?;
        self.closed = true;
        Ok(())
    }
    pub fn flush(&mut self) -> Result<()> {
        if self.closed {
            return Err(anyhow!("File is already closed"));
        }
        let fs = &mut Fs::get_mut().fstab[self.fs_index].fs;
        fs.flush(&mut self.node)
    }
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.closed {
            return Err(anyhow!("File is already closed"));
        }
        let fs = &mut Fs::get_mut().fstab[self.fs_index].fs;
        fs.read(&mut self.node, buf)
    }
    pub fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if self.closed {
            return Err(anyhow!("File is already closed"));
        }
        let fs = &mut Fs::get_mut().fstab[self.fs_index].fs;
        fs.write(&mut self.node, buf)
    }
    #[allow(unused)]
    pub fn seek(&mut self, pos: isize, whence: Whence) -> Result<isize> {
        if self.closed {
            return Err(anyhow!("File is already closed"));
        }
        let fs = &mut Fs::get_mut().fstab[self.fs_index].fs;
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
    pub fn fstab() -> &'static mut [FsEntry] {
        Fs::get_mut().fstab
    }
    pub fn cwd() -> String {
        Fs::get_mut().cwd.clone()
    }
    pub fn init(fstab: &'static mut [FsEntry]) -> Result<()> {
        Fs::get_mut().fstab = fstab;

        for entry in Fs::get_mut().fstab.iter_mut() {
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
        for entry in Fs::get_mut().fstab.iter_mut() {
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
        for (index, entry) in Fs::get_mut().fstab.iter_mut().enumerate() {
            if path.starts_with(entry.mount_point) {
                let path = &path[entry.mount_point.len()..];
                let path = if path.is_empty() { "/" } else { path };
                return Ok((entry.fs, index, path));
            }
        }
        Err(anyhow!("Filesystem not found for path: {}", path))
    }
    fn mount_point_to_fs(mount_point: &str) -> Result<&'static mut dyn FsHandle> {
        for entry in Fs::get_mut().fstab.iter_mut() {
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
            for entry in Fs::get_mut().fstab.iter() {
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
        for entry in Fs::get_mut().fstab.iter_mut() {
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
        Fs::get_mut().cwd = path.to_string();
        Ok(())
    }
    pub fn get_cwd() -> String {
        Fs::get_mut().cwd.clone()
    }
    pub fn to_absolute_path(path: &str) -> String {
        // println!("1> {}", path);
        let path = if path.starts_with('/') {
            path.to_string()
        } else {
            let cwd = Fs::get_mut().cwd.clone();
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
}
