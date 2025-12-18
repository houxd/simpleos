use simpleos::alloc::boxed::Box;
use simpleos::console::BuiltinCmds;
use simpleos::console::Console;
use simpleos::console::ConsoleDriver;
use simpleos::driver::cpu::CpuDriver;
use simpleos::driver::lazy_init::LazyInit;
use simpleos::driver::systick::SysTickDriver;
use simpleos::driver::Driver;
use simpleos::executor::Executor;
use simpleos::executor::ExitCode;
use simpleos::singleton;
use simpleos::sys::Device;
use simpleos::sys::SimpleOs;
use simpleos::Result;
use termion::raw::RawTerminal;
use std::io::{stdin, Read, Write};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;
use termion::raw::IntoRawMode;

struct CpuEmulate;

impl Driver for CpuEmulate {
    fn driver_init(&mut self) -> Result<()> {
        Ok(())
    }

    fn driver_deinit(&mut self) -> Result<()> {
        Ok(())
    }
}

impl CpuDriver for CpuEmulate {
    fn cpu_reset(&mut self) -> ! {
        BoardEmulate::get_mut().console0.get().unwrap().restore_terminal();
        panic!("System reset called in emulation.");
    }
    fn cpu_panic(&mut self, panic_info: String) -> ! {
        BoardEmulate::get_mut().console0.get().unwrap().restore_terminal();
        panic!("Panic: {}", panic_info);
    }
}

struct SysTickEmulate;

impl Driver for SysTickEmulate {
    fn driver_init(&mut self) -> Result<()> {
        Ok(())
    }

    fn driver_deinit(&mut self) -> Result<()> {
        Ok(())
    }
}

impl SysTickDriver for SysTickEmulate {
    fn get_system_ms(&self) -> u32 {
        static mut BOOT_TIMESTAMP: u128 = 0;

        fn get_ms_from_unix_epoch() -> u128 {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        }

        unsafe {
            let current_time = get_ms_from_unix_epoch();
            (current_time - BOOT_TIMESTAMP) as u32
        }
    }
}

struct ConsoleIOEmulate {
    rx: Receiver<u8>,
    raw_term: Arc<Mutex<Option<RawTerminal<std::io::Stdout>>>>,
}

impl ConsoleIOEmulate {
    fn new() -> Self {
        let (tx, rx) = channel();
        let raw_term = Arc::new(Mutex::new(None));
        let raw_term_clone = raw_term.clone();

        thread::spawn(move || {
            let mut stdin = stdin();
            let raw = std::io::stdout().into_raw_mode().unwrap();
            *raw_term_clone.lock().unwrap() = Some(raw);

            let mut buffer = [0u8; 1];
            loop {
                if stdin.read_exact(&mut buffer).is_ok() {
                    if buffer[0] == 3 {
                        if let Some(pid) = Console::get_mut().fg_task_id {
                            Executor::kill(pid);
                        }
                    }

                    if tx.send(buffer[0]).is_err() {
                        break;
                    }
                }
            }
            // 线程结束时自动 drop raw_term，恢复终端
        });

        ConsoleIOEmulate { rx, raw_term }
    }

    fn restore_terminal(&mut self) {
        // 显式恢复终端
        *self.raw_term.lock().unwrap() = None;
    }
}

impl Driver for ConsoleIOEmulate {
    fn driver_init(&mut self) -> Result<()> {
        Ok(())
    }
    fn driver_deinit(&mut self) -> Result<()> {
        Ok(())
    }
}

impl ConsoleDriver for ConsoleIOEmulate {
    fn console_getc(&mut self) -> Option<u8> {
        let c = match self.rx.try_recv() {
            Ok(byte) => Some(byte),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        };
        c
    }

    fn console_putc(&mut self, byte: u8) -> bool {
        print!("{}", byte as char);
        true
    }

    fn console_flush(&mut self) {
        std::io::stdout().flush().unwrap();
    }
}

struct BoardEmulate {
    cpu0: LazyInit<CpuEmulate>,
    systick0: LazyInit<SysTickEmulate>,
    console0: LazyInit<ConsoleIOEmulate>,
}

singleton!(BoardEmulate {
    cpu0: LazyInit::new(|| CpuEmulate {}),
    systick0: LazyInit::new(|| SysTickEmulate {}),
    console0: LazyInit::new(|| ConsoleIOEmulate::new()),
});

impl Device for BoardEmulate {
    fn get_cpu(&self) -> &'static mut dyn simpleos::driver::cpu::CpuDriver {
        BoardEmulate::get_mut().cpu0.get_or_init()
    }

    fn get_console(&self) -> &'static mut dyn ConsoleDriver {
        BoardEmulate::get_mut().console0.get_or_init()
    }

    fn get_systick(&self) -> &'static mut dyn SysTickDriver {
        BoardEmulate::get_mut().systick0.get_or_init()
    }
}

async fn init() -> ExitCode {
    let pid = Executor::spawn("console", Box::pin(Console::start()));
    Executor::wait(pid).await;
    0
}

fn main() {
    SimpleOs::init(BoardEmulate::get_mut());
    Console::add_commands(BuiltinCmds);
    Executor::spawn("init", Box::pin(init()));
    Executor::run();
}
