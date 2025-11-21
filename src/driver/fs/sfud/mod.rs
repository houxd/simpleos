use crate::{
    bindings,
    // driver::{stm32::Stm32, stm32_gpio::Stm32Gpio},
    println,
    //  unsafe_get_mut,
};

// const SPI1_CS: Stm32Gpio = Stm32Gpio::new('A', 4);
pub struct Sfud;

impl Sfud {
    pub fn init() {
        unsafe {
            bindings::sfud_init();
        }
    }

    /*
       sfud_err sfud_spi_port_init(sfud_flash *flash)
    */
    #[no_mangle]
    extern "C" fn sfud_spi_port_init(flash: *mut bindings::sfud_flash) -> bindings::sfud_err {
        unsafe {
            (*flash).spi.wr = Some(Self::sfud_spi_write_read);
            (*flash).spi.lock = None;
            (*flash).spi.unlock = None;
            (*flash).spi.user_data = core::ptr::null_mut();
            (*flash).retry.delay = Some(Self::sfud_spi_delay);
            (*flash).retry.times = 60 * 1000;
        }
        bindings::sfud_err_SFUD_SUCCESS
    }

    /*
       sfud_err spi_write_read(const sfud_spi *spi, const uint8_t *write_buf, size_t write_size, uint8_t *read_buf,
    */
    extern "C" fn sfud_spi_write_read(
        spi: *const bindings::sfud_spi,
        write_buf: *const u8,
        write_size: usize,
        read_buf: *mut u8,
        read_size: usize,
    ) -> bindings::sfud_err {
        // unsafe {
        //     let spi_name = core::ffi::CStr::from_ptr((*spi).name).to_str().unwrap_or("Invalid UTF-8");
        //     match spi_name {
        //         "spi0" => {
        //             for _ in 0..10 {
        //                 let spi_status = bindings::HAL_SPI_GetState(unsafe_get_mut!(bindings::hspi1));
        //                 if spi_status == bindings::HAL_SPI_StateTypeDef_HAL_SPI_STATE_READY {
        //                     break;
        //                 }
        //                 Stm32::delay(1);
        //             }
        //             SPI1_CS.write(false);
        //             if write_size > 0 {
        //                 bindings::HAL_SPI_Transmit(
        //                     unsafe_get_mut!(bindings::hspi1),
        //                     write_buf as *const u8,
        //                     write_size as u16,
        //                     bindings::HAL_MAX_DELAY,
        //                 );
        //             }
        //             if read_size > 0 {
        //                 bindings::HAL_SPI_Receive(
        //                     unsafe_get_mut!(bindings::hspi1),
        //                     read_buf as *mut u8,
        //                     read_size as u16,
        //                     bindings::HAL_MAX_DELAY,
        //                 );
        //             }
        //             SPI1_CS.write(true);
        //         }
        //         _ => {
        //             // spi1,spi2 not implemented
        //             return bindings::sfud_err_SFUD_ERR_NOT_FOUND;
        //         }
        //     }
        // }
        bindings::sfud_err_SFUD_SUCCESS
    }

    /*
       void spi_delay(void)
    */
    extern "C" fn sfud_spi_delay() {
        // Stm32::delay(1);
    }

    #[no_mangle]
    extern "C" fn sfud_print(content: *const i8) {
        println!("SFUD: {}", unsafe {
            core::ffi::CStr::from_ptr(content as _).to_str().unwrap_or("Invalid UTF-8")
        });
    }
}
