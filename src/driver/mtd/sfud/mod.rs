use core::pin::Pin;
use alloc::boxed::Box;
use crate::console::println;
use crate::driver::mtd::MtdDriver;
use crate::driver::spi::SpiDriver;
use crate::driver::Driver;
use crate::sys::delay_ms;
use crate::{bindings};

pub struct Sfud {
    flash: Pin<Box<bindings::sfud_flash>>,
}

impl Sfud {
    pub fn new(spi: &'static mut dyn SpiDriver) -> Self {
        let mut flash = Box::pin(bindings::sfud_flash::default());
        flash.name = "default\0".as_ptr() as *mut _;
        flash.spi.name = "default\0".as_ptr() as *mut _;

        // let spi_ptr = Box::leak(Box::new(spi)) as *mut &'static mut dyn SpiDriver;
        // flash.spi.user_data = spi_ptr as *mut core::ffi::c_void;

        let spi_ptr = spi as *mut dyn SpiDriver;
        let boxed = Box::new(spi_ptr);
        flash.spi.user_data = Box::into_raw(boxed) as *mut core::ffi::c_void;

        Self { flash }
    }
}

impl Drop for Sfud {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.flash.spi.user_data as *mut *mut dyn SpiDriver;
            let _ = Box::from_raw(ptr);
        }
    }
}

impl Driver for Sfud {
    fn driver_init(&mut self) -> anyhow::Result<()> {
        unsafe {
            let flash_mut = Pin::get_unchecked_mut(self.flash.as_mut());
            bindings::sfud_device_init(flash_mut);
        }
        Ok(())
    }
    fn driver_deinit(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl MtdDriver for Sfud {
    fn mtd_read(&mut self, addr: u32, buffer: &mut [u8]) -> anyhow::Result<()> {
        unsafe {
            let flash = bindings::sfud_get_device(0);
            bindings::sfud_read(flash, addr, buffer.len(), buffer.as_mut_ptr());
        }
        Ok(())
    }
    fn mtd_write(&mut self, addr: u32, data: &[u8]) -> anyhow::Result<()> {
        unsafe {
            let flash = bindings::sfud_get_device(0);
            bindings::sfud_write(flash, addr, data.len(), data.as_ptr());
        }
        Ok(())
    }
    fn mtd_erase(&mut self, addr: u32, size: u32) -> anyhow::Result<()> {
        unsafe {
            let flash = bindings::sfud_get_device(0);
            bindings::sfud_erase(flash, addr, size as usize);
        }
        Ok(())
    }

    // fn block_size(&self) -> u32 {
    //     // 这里需要获取实际的擦除块大小
    //     4096
    //     // unsafe {
    //     //     let flash = bindings::sfud_get_device(0);
    //     //     (*flash).chip.erase_gran
    //     // }
    // }

    // fn total_size(&self) -> u32 {
    //     unsafe {
    //         let flash = bindings::sfud_get_device(0);
    //         (*flash).chip.capacity
    //     }
    // }
}

#[no_mangle]
extern "C" fn sfud_spi_port_init(flash: *mut bindings::sfud_flash) -> bindings::sfud_err {
    unsafe {
        (*flash).spi.wr = Some(sfud_spi_write_read);
        (*flash).spi.lock = None;
        (*flash).spi.unlock = None;
        (*flash).retry.delay = Some(sfud_spi_delay);
        (*flash).retry.times = 60 * 1000;
    }
    bindings::sfud_err_SFUD_SUCCESS
}

#[no_mangle]
extern "C" fn sfud_spi_write_read(
    flash_spi: *const bindings::sfud_spi, // currently unused, only one SPI flash is supported
    write_buf: *const u8,
    write_size: usize,
    read_buf: *mut u8,
    read_size: usize,
) -> bindings::sfud_err {
    unsafe {
        // let spi = &mut **((*flash_spi).user_data as *mut &'static mut dyn SpiDriver);
        let spi_ptr_ptr = (*flash_spi).user_data as *mut *mut dyn SpiDriver;
        let spi = &mut **spi_ptr_ptr;

        spi.spi_cs_activate();
        if write_size > 0 && !write_buf.is_null() {
            if let Err(e) = spi.spi_write(core::slice::from_raw_parts(write_buf, write_size)) {
                println!("SFUD ERR: {:?}", e);
                spi.spi_cs_deactivate();
                return bindings::sfud_err_SFUD_ERR_WRITE;
            }
        }
        if read_size > 0 && !read_buf.is_null() {
            if let Err(e) = spi.spi_read(core::slice::from_raw_parts_mut(read_buf, read_size)) {
                println!("SFUD ERR: {:?}", e);
                spi.spi_cs_deactivate();
                return bindings::sfud_err_SFUD_ERR_READ;
            }
        }
        spi.spi_cs_deactivate();
    }
    bindings::sfud_err_SFUD_SUCCESS
}

#[no_mangle]
extern "C" fn sfud_spi_delay() {
    delay_ms(1);
}

#[no_mangle]
extern "C" fn sfud_print(content: *const i8) {
    println!("SFUD: {}", unsafe {
        core::ffi::CStr::from_ptr(content as _)
            .to_str()
            .unwrap_or("INV UTF8")
    });
}
