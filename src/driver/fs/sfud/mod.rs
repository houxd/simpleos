use crate::console::print;
use crate::driver::spi::SpiDriver;
use crate::driver::Driver;
use crate::{bindings, singleton};

struct SfudGlobalData {
    pub spi: Option<*mut dyn SpiDriver>,
}

singleton!(SfudGlobalData { spi: None });

impl SfudGlobalData {
    pub fn spi() -> &'static mut dyn SpiDriver {
        unsafe { &mut *SfudGlobalData::mut_ref().spi.unwrap() }
    }
}

pub struct Sfud {
    spi: &'static mut dyn SpiDriver,
}

impl Sfud {
    pub const fn new(spi: &'static mut dyn SpiDriver) -> Self {
        Self { spi }
    }
}

impl Driver for Sfud {
    fn driver_init(&mut self) -> anyhow::Result<()> {
        unsafe {
            SfudGlobalData::mut_ref().spi = Some(self.spi as *mut dyn SpiDriver);
            bindings::sfud_init();
        }
        Ok(())
    }
    fn driver_deinit(&mut self) -> anyhow::Result<()> {
        SfudGlobalData::mut_ref().spi = None;
        Ok(())
    }
}


// sfud_err sfud_spi_port_init(sfud_flash *flash)
#[no_mangle]
extern "C" fn sfud_spi_port_init(flash: *mut bindings::sfud_flash) -> bindings::sfud_err {
    unsafe {
        (*flash).spi.wr = Some(sfud_spi_write_read);
        (*flash).spi.lock = None;
        (*flash).spi.unlock = None;
        (*flash).spi.user_data = core::ptr::null_mut();
        (*flash).retry.delay = Some(sfud_spi_delay);
        (*flash).retry.times = 60 * 1000;
    }
    bindings::sfud_err_SFUD_SUCCESS
}


// sfud_err spi_write_read(const sfud_spi *spi, const uint8_t *write_buf, size_t write_size, uint8_t *read_buf, size_t read_size)
#[no_mangle]
extern "C" fn sfud_spi_write_read(
    _spi: *const bindings::sfud_spi, // currently unused, only one SPI flash is supported
    write_buf: *const u8,
    write_size: usize,
    read_buf: *mut u8,
    read_size: usize,
) -> bindings::sfud_err {
    SfudGlobalData::spi().spi_cs_activate();
    if let Err(e) = SfudGlobalData::spi().spi_write(unsafe { core::slice::from_raw_parts(write_buf, write_size) }) {
        print!("SFUD ERR: {:?}\n", e);
        SfudGlobalData::spi().spi_cs_deactivate();
        return bindings::sfud_err_SFUD_ERR_WRITE;
    }
    if let Err(e) = SfudGlobalData::spi().spi_read(unsafe { core::slice::from_raw_parts_mut(read_buf, read_size) }) {
        print!("SFUD ERR: {:?}\n", e);
        SfudGlobalData::spi().spi_cs_deactivate();
        return bindings::sfud_err_SFUD_ERR_READ;
    }
    SfudGlobalData::spi().spi_cs_deactivate();
    bindings::sfud_err_SFUD_SUCCESS
}

// void spi_delay(void)
#[no_mangle]
extern "C" fn sfud_spi_delay() {
    // Stm32::delay(1);
}

#[no_mangle]
extern "C" fn sfud_print(content: *const i8) {
    print!("SFUD: {}", unsafe {
        core::ffi::CStr::from_ptr(content as _)
            .to_str()
            .unwrap_or("INV UTF8")
    });
}
