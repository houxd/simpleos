use crate::{
    bindings::*,
    driver::{
        fs::{DirEntry, FileHandle, FsHandle, FsInfo, Whence},
        mtd::MtdDriver,
    },
};
use alloc::{boxed::Box, vec::Vec};
use anyhow::{anyhow, Result};
use core::any::Any;

pub struct LittleFs {
    lfs: lfs_t,
    lfs_cfg: lfs_config,
}

impl LittleFs {
    pub fn new(mtd: &'static mut dyn MtdDriver) -> Self {
        // 构建littlefs配置
        let mut lfs_cfg = lfs_config::default();
        lfs_cfg.read = Some(lfs_read);
        lfs_cfg.prog = Some(lfs_prog);
        lfs_cfg.erase = Some(lfs_erase);
        lfs_cfg.sync = Some(lfs_sync);
        lfs_cfg.read_size = 256;
        lfs_cfg.prog_size = 256;
        // lfs_cfg.block_size = mtd.block_size();
        // lfs_cfg.block_count = mtd.total_size() / mtd.block_size();
        lfs_cfg.block_size = 4096;
        lfs_cfg.block_count = (8 * 1024 * 1024) / 4096;
        lfs_cfg.cache_size = 256 * 8;
        lfs_cfg.lookahead_size = 128;
        lfs_cfg.block_cycles = 500;

        let mtd_ptr = Box::leak(Box::new(mtd)) as *mut &'static mut dyn MtdDriver;
        lfs_cfg.context = mtd_ptr as *mut core::ffi::c_void;

        LittleFs {
            lfs: lfs_t::default(),
            lfs_cfg,
        }
    }
}

impl Drop for LittleFs {
    fn drop(&mut self) {
        unsafe {
            // 释放 MTD 驱动的 Box
            let mtd_ptr = self.lfs_cfg.context as *mut &'static mut dyn MtdDriver;
            let _ = Box::from_raw(mtd_ptr);
        }
    }
}

#[no_mangle]
extern "C" fn lfs_read(
    c: *const lfs_config,
    block: lfs_block_t,
    off: lfs_off_t,
    buffer: *mut core::ffi::c_void,
    size: lfs_size_t,
) -> core::ffi::c_int {
    unsafe {
        let mtd = &mut **((*c).context as *mut &'static mut dyn MtdDriver);
        let addr = block * (*c).block_size + off;
        let buf_slice = core::slice::from_raw_parts_mut(buffer as *mut u8, size as usize);
        match mtd.mtd_read(addr as u32, buf_slice) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}
#[no_mangle]
extern "C" fn lfs_prog(
    c: *const lfs_config,
    block: lfs_block_t,
    off: lfs_off_t,
    buffer: *const core::ffi::c_void,
    size: lfs_size_t,
) -> core::ffi::c_int {
    unsafe {
        let mtd = &mut **((*c).context as *mut &'static mut dyn MtdDriver);
        let addr = block * (*c).block_size + off;
        let buf_slice = core::slice::from_raw_parts(buffer as *const u8, size as usize);
        match mtd.mtd_write(addr as u32, buf_slice) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}
#[no_mangle]
extern "C" fn lfs_erase(c: *const lfs_config, block: lfs_block_t) -> core::ffi::c_int {
    unsafe {
        let mtd = &mut **((*c).context as *mut &'static mut dyn MtdDriver);
        let addr = block * (*c).block_size;
        match mtd.mtd_erase(addr as u32, (*c).block_size as u32) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}
#[no_mangle]
extern "C" fn lfs_sync(_c: *const lfs_config) -> core::ffi::c_int {
    0
}

impl FsHandle for LittleFs {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn info(&mut self) -> Result<Box<dyn FsInfo>> {
        unsafe {
            let total = (self.lfs_cfg.block_count * self.lfs_cfg.block_size) as isize;
            let free = lfs_fs_size(&mut self.lfs) as isize;
            let used = total - free;
            Ok(alloc::boxed::Box::new(LittleFsInfo { total, used, free }))
        }
    }

    fn mount(&mut self) -> Result<()> {
        unsafe {
            let res = lfs_mount(&mut self.lfs, &self.lfs_cfg);
            if res != 0 {
                Err(anyhow!("LittleFS mount failed: {}", res))
            } else {
                Ok(())
            }
        }
    }

    fn unmount(&mut self) -> Result<()> {
        unsafe {
            let res = lfs_unmount(&mut self.lfs);
            if res != 0 {
                Err(anyhow!("LittleFS unmount failed: {}", res))
            } else {
                Ok(())
            }
        }
    }

    fn format(&mut self) -> Result<()> {
        unsafe {
            let res = lfs_format(&mut self.lfs, &self.lfs_cfg);
            if res != 0 {
                Err(anyhow!("LittleFS format failed: {}", res))
            } else {
                Ok(())
            }
        }
    }

    fn mkdir(&mut self, path: &str) -> Result<()> {
        unsafe {
            let c_path = alloc::ffi::CString::new(path).unwrap();
            let res = lfs_mkdir(&mut self.lfs, c_path.as_ptr());
            if res != 0 {
                Err(anyhow!("LittleFS mkdir failed: {}", res))
            } else {
                Ok(())
            }
        }
    }

    fn unlink(&mut self, path: &str) -> Result<()> {
        unsafe {
            let c_filename = alloc::ffi::CString::new(path).unwrap();
            let res = lfs_remove(&mut self.lfs, c_filename.as_ptr());
            if res != 0 {
                Err(anyhow!("LittleFS unlink failed: {}", res))
            } else {
                Ok(())
            }
        }
    }

    fn rename(&mut self, old_path: &str, new_path: &str) -> Result<()> {
        unsafe {
            let c_old_path = alloc::ffi::CString::new(old_path).unwrap();
            let c_new_path = alloc::ffi::CString::new(new_path).unwrap();
            let res = lfs_rename(&mut self.lfs, c_old_path.as_ptr(), c_new_path.as_ptr());
            if res != 0 {
                Err(anyhow!("LittleFS rename failed: {}", res))
            } else {
                Ok(())
            }
        }
    }

    fn stat(&mut self, path: &str) -> Result<Box<dyn DirEntry>> {
        unsafe {
            let c_filename = alloc::ffi::CString::new(path).unwrap();
            let mut stat = Box::new(lfs_info::default());
            let stat_ptr = stat.as_mut() as *mut lfs_info;
            let res = lfs_stat(&mut self.lfs, c_filename.as_ptr(), stat_ptr);
            if res != 0 {
                Err(anyhow!("LittleFS stat failed: {}", res))
            } else {
                Ok(stat as Box<dyn DirEntry>)
            }
        }
    }

    fn readdir(&mut self, path: &str) -> Result<Vec<Box<dyn DirEntry>>> {
        unsafe {
            let c_path = alloc::ffi::CString::new(path).unwrap();
            let mut dir = lfs_dir::default();
            let res = lfs_dir_open(&mut self.lfs, &mut dir, c_path.as_ptr());
            if res != 0 {
                return Err(anyhow!("LittleFS readdir open failed: {}", res));
            }
            let mut entries: Vec<Box<dyn DirEntry>> = Vec::new();
            loop {
                let mut info = Box::new(lfs_info::default());
                let res = lfs_dir_read(&mut self.lfs, &mut dir, info.as_mut());
                if res < 0 {
                    lfs_dir_close(&mut self.lfs, &mut dir);
                    return Err(anyhow!("LittleFS readdir read failed: {}", res));
                }
                if res == 0 {
                    break;
                }
                entries.push(info as Box<dyn DirEntry>);
            }
            lfs_dir_close(&mut self.lfs, &mut dir);
            Ok(entries)
        }
    }

    fn sync(&mut self) -> Result<()> {
        Ok(())
    }

    fn open(&mut self, path: &str, mode: &str) -> Result<Box<dyn FileHandle>> {
        unsafe {
            let mut file = Box::new(lfs_file::default());
            let file_ptr = file.as_mut() as *mut lfs_file;
            let c_path = alloc::ffi::CString::new(path).unwrap();
            let flags = match mode {
                "r" => lfs_open_flags_LFS_O_RDONLY,
                "r+" => lfs_open_flags_LFS_O_RDWR,
                "w" => {
                    lfs_open_flags_LFS_O_WRONLY
                        | lfs_open_flags_LFS_O_CREAT
                        | lfs_open_flags_LFS_O_TRUNC
                }
                "w+" => {
                    lfs_open_flags_LFS_O_RDWR
                        | lfs_open_flags_LFS_O_CREAT
                        | lfs_open_flags_LFS_O_TRUNC
                }
                "a" => {
                    lfs_open_flags_LFS_O_WRONLY
                        | lfs_open_flags_LFS_O_CREAT
                        | lfs_open_flags_LFS_O_APPEND
                }
                "a+" => {
                    lfs_open_flags_LFS_O_RDWR
                        | lfs_open_flags_LFS_O_CREAT
                        | lfs_open_flags_LFS_O_APPEND
                }
                _ => return Err(anyhow!("Invalid mode: {}", mode)),
            };
            let res = lfs_file_open(&mut self.lfs, file_ptr, c_path.as_ptr(), flags as _);
            if res != 0 {
                Err(anyhow!("LittleFS open file failed: {}", res))
            } else {
                Ok(file as Box<dyn FileHandle>)
            }
        }
    }

    fn close(&mut self, file_node: &mut Box<dyn FileHandle>) -> Result<()> {
        unsafe {
            let lfs_file = file_node
                .as_any_mut()
                .downcast_mut::<lfs_file>()
                .ok_or_else(|| anyhow!("Invalid file type"))?;
            let res = lfs_file_close(&mut self.lfs, lfs_file as *mut lfs_file);
            if res != 0 {
                Err(anyhow!("LittleFS clone file failed: {}", res))
            } else {
                Ok(())
            }
        }
    }

    fn flush(&mut self, file_node: &mut Box<dyn FileHandle>) -> Result<()> {
        unsafe {
            let lfs_file = file_node
                .as_any_mut()
                .downcast_mut::<lfs_file>()
                .ok_or_else(|| anyhow!("Invalid file type"))?;
            let res = lfs_file_sync(&mut self.lfs, lfs_file as *mut lfs_file);
            if res != 0 {
                Err(anyhow!("LittleFS sync file failed: {}", res))
            } else {
                Ok(())
            }
        }
    }

    fn read(&mut self, file_handle: &mut Box<dyn FileHandle>, buf: &mut [u8]) -> Result<usize> {
        unsafe {
            let lfs_file = file_handle
                .as_any_mut()
                .downcast_mut::<lfs_file>()
                .ok_or_else(|| anyhow!("Invalid file type"))?;
            let res = lfs_file_read(
                &mut self.lfs,
                lfs_file as *mut lfs_file,
                buf.as_mut_ptr() as *mut core::ffi::c_void, // 转换为 c_void
                buf.len() as u32,                           // 转换为 u32
            );
            if res < 0 {
                Err(anyhow!("LittleFS read file failed: {}", res))
            } else {
                Ok(res as usize)
            }
        }
    }

    fn write(&mut self, file_handle: &mut Box<dyn FileHandle>, buf: &[u8]) -> Result<usize> {
        unsafe {
            let lfs_file = file_handle
                .as_any_mut()
                .downcast_mut::<lfs_file>()
                .ok_or_else(|| anyhow!("Invalid file type"))?;
            let res = lfs_file_write(
                &mut self.lfs,
                lfs_file as *mut lfs_file,
                buf.as_ptr() as *const core::ffi::c_void, // 转换为 c_void
                buf.len() as u32,                         // 转换为 u32
            );
            if res < 0 {
                Err(anyhow!("LittleFS write file failed: {}", res))
            } else {
                Ok(res as usize)
            }
        }
    }

    fn seek(
        &mut self,
        file: &mut Box<dyn FileHandle>,
        pos: isize,
        whence: Whence,
    ) -> Result<isize> {
        unsafe {
            let lfs_file = file
                .as_any_mut()
                .downcast_mut::<lfs_file>()
                .ok_or_else(|| anyhow!("Invalid file type"))?;
            let res = lfs_file_seek(
                &mut self.lfs,
                lfs_file as *mut lfs_file,
                pos as lfs_soff_t,
                whence as i32,
            );
            if res < 0 {
                Err(anyhow!("LittleFS seek file failed: {}", res))
            } else {
                Ok(res as isize)
            }
        }
    }
}

struct LittleFsInfo {
    total: isize,
    used: isize,
    free: isize,
}

impl FsInfo for LittleFsInfo {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn total(&self) -> isize {
        self.total
    }
    fn used(&self) -> isize {
        self.used
    }
    fn free(&self) -> isize {
        self.free
    }
}

impl FileHandle for lfs_file {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl DirEntry for lfs_info {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn name(&self) -> &str {
        unsafe {
            let c_str = core::ffi::CStr::from_ptr(self.name.as_ptr());
            c_str.to_str().unwrap_or("")
        }
    }
    fn is_file(&self) -> bool {
        self.type_ == lfs_type_LFS_TYPE_REG as u8
    }
    fn is_dir(&self) -> bool {
        self.type_ == lfs_type_LFS_TYPE_DIR as u8
    }
    fn size(&self) -> usize {
        self.size as usize
    }
}
