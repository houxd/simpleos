use crate::bindings;
use crate::println;
use crate::driver::mtd::MtdDriver;
use crate::driver::spi::SpiDriver;
use crate::driver::Driver;
use crate::sys;
use alloc::boxed::Box;
use core::pin::Pin;

pub struct Sfud {
    flash: Pin<Box<bindings::sfud_flash>>,
    spi: &'static mut dyn SpiDriver,
}

impl Sfud {
    pub fn new(spi: &'static mut dyn SpiDriver) -> Self {
        let mut flash = Box::pin(bindings::sfud_flash::default());
        flash.name = "default\0".as_ptr() as *mut _;
        flash.spi.name = "default\0".as_ptr() as *mut _;
        flash.spi.user_data = core::ptr::null_mut();
        Self { flash, spi }
    }
    #[no_mangle]
    extern "C" fn sfud_spi_port_init(flash: *mut bindings::sfud_flash) -> bindings::sfud_err {
        unsafe {
            (*flash).spi.wr = Some(Self::sfud_spi_write_read);
            (*flash).spi.lock = None;
            (*flash).spi.unlock = None;
            (*flash).retry.delay = Some(Self::sfud_spi_delay);
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
            // 从 user_data 获取 Sfud 的裸指针
            let sfud_ptr = (*flash_spi).user_data as *mut Sfud;
            if sfud_ptr.is_null() {
                return bindings::sfud_err_SFUD_ERR_NOT_FOUND;
            }
            let sfud = &mut *sfud_ptr;

            sfud.spi.spi_cs_activate();
            if write_size > 0 && !write_buf.is_null() {
                if let Err(e) = sfud
                    .spi
                    .spi_write(core::slice::from_raw_parts(write_buf, write_size))
                {
                    println!("SFUD ERR: {:?}", e);
                    sfud.spi.spi_cs_deactivate();
                    return bindings::sfud_err_SFUD_ERR_WRITE;
                }
            }
            if read_size > 0 && !read_buf.is_null() {
                if let Err(e) = sfud
                    .spi
                    .spi_read(core::slice::from_raw_parts_mut(read_buf, read_size))
                {
                    println!("SFUD ERR: {:?}", e);
                    sfud.spi.spi_cs_deactivate();
                    return bindings::sfud_err_SFUD_ERR_READ;
                }
            }
            sfud.spi.spi_cs_deactivate();
        }
        bindings::sfud_err_SFUD_SUCCESS
    }

    #[no_mangle]
    extern "C" fn sfud_spi_delay() {
        sys::delay_ms(1);
    }

    #[no_mangle]
    extern "C" fn sfud_print(content: *const i8) {
        println!("SFUD: {}", unsafe {
            core::ffi::CStr::from_ptr(content as _)
                .to_str()
                .unwrap_or("INV UTF8")
        });
    }
}

impl Driver for Sfud {
    fn driver_init(&mut self) -> anyhow::Result<()> {
        unsafe {
            let self_ptr = self as *mut Self;
            let flash_mut = Pin::get_unchecked_mut(self.flash.as_mut());
            flash_mut.spi.user_data = self_ptr as *mut core::ffi::c_void;
            let res = bindings::sfud_device_init(flash_mut);
            if res != bindings::sfud_err_SFUD_SUCCESS {
                return Err(anyhow::anyhow!("SFUD device init failed: {}", res));
            }
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
            let flash = self.flash.as_mut().get_unchecked_mut();
            bindings::sfud_read(flash, addr, buffer.len(), buffer.as_mut_ptr());
        }
        Ok(())
    }
    fn mtd_write(&mut self, addr: u32, data: &[u8]) -> anyhow::Result<()> {
        unsafe {
            let flash = self.flash.as_mut().get_unchecked_mut();
            bindings::sfud_write(flash, addr, data.len(), data.as_ptr());
        }
        Ok(())
    }
    fn mtd_erase(&mut self, addr: u32, size: u32) -> anyhow::Result<()> {
        unsafe {
            let flash = self.flash.as_mut().get_unchecked_mut();
            bindings::sfud_erase(flash, addr, size as usize);
        }
        Ok(())
    }

    fn size(&mut self) -> u32 {
        unsafe {
            let flash = self.flash.as_mut().get_unchecked_mut();
            ((*flash).chip.capacity) as u32
        }
    }

    fn erase_size(&mut self) -> u32 {
        unsafe {
            let flash = self.flash.as_mut().get_unchecked_mut();
            ((*flash).chip.erase_gran) as u32
        }
    }

    fn write_size(&mut self) -> u32 {
        256
    }
}
