use crate::console::cmd::CmdParse;
use crate::console::console_driver::ConsoleDriver;
use crate::driver::Driver;
use crate::sys::{sleep_ms, yield_now, Executor};
use crate::util::RingBuf;
use crate::{join, println, select, singleton};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use heapless::Vec as HeaplessVec;

const HISTORY_SIZE: usize = 10; // 历史记录最大条数
const LINE_BUFFER_SIZE: usize = 512; // 每行最大字符数

/// 控制台设备默认的空实现
struct DummyConsole;
impl Driver for DummyConsole {
    fn driver_init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn driver_deinit(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
impl ConsoleDriver for DummyConsole {
    fn console_getc(&mut self) -> Option<u8> {
        None
    }
    fn console_putc(&mut self, _byte: u8) -> bool {
        true
    }
    fn console_flush(&mut self) {}
}

singleton!(DummyConsole {});

#[derive(Debug, Clone, Copy)]
enum EscapeState {
    Normal,
    Escape,
    Bracket,
}

struct SignalHandler {
    f: Box<dyn Fn(u8) -> Pin<Box<dyn Future<Output = ()>>>>,
}

impl SignalHandler {
    // 从异步函数创建
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(u8) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        SignalHandler {
            f: Box::new(move |sig| Box::pin(f(sig))),
        }
    }

    // 从同步闭包创建(返回异步块)
    pub fn from_sync<F>(f: F) -> Self
    where
        F: Fn(u8) + 'static,
    {
        SignalHandler {
            f: Box::new(move |sig| {
                f(sig);
                Box::pin(async {})
            }),
        }
    }

    fn call(&self, sig: u8) -> Pin<Box<dyn Future<Output = ()>>> {
        (self.f)(sig)
    }
}

pub struct Console {
    pub dev: &'static mut dyn ConsoleDriver,
    prompt: String,
    // 历史记录
    history: HeaplessVec<HeaplessVec<u8, LINE_BUFFER_SIZE>, HISTORY_SIZE>,
    history_index: Option<usize>,
    // 当前编辑状态
    current_line: HeaplessVec<u8, LINE_BUFFER_SIZE>,
    cursor_pos: usize,
    // ANSI转义序列状态
    escape_state: EscapeState,
    signal_interrupt: RingBuf<u8, 3>, // 用ringbuf更好, 中断要访问
    signal_handler: SignalHandler,
}

singleton!(Console {
    dev: DummyConsole::ref_mut(),
    prompt: String::from("> "),
    history: HeaplessVec::new(),
    history_index: None,
    current_line: HeaplessVec::new(),
    cursor_pos: 0,
    escape_state: EscapeState::Normal,
    signal_interrupt: RingBuf::new(),
    signal_handler: SignalHandler::new(default_signal_handler),
});

async fn default_signal_handler(sig: u8) {
    if sig == 3 {
        Executor::exit().await;
    }
}

#[allow(unused)]
impl Console {
    pub fn init(dev: &'static mut dyn ConsoleDriver) {
        let console = Console::ref_mut();
        console.dev = dev;
    }

    #[inline]
    pub fn device() -> &'static mut dyn ConsoleDriver {
        Console::ref_mut().dev
    }

    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = String::from(prompt);
    }

    fn show_prompt(&mut self) {
        self.dev.console_write(self.prompt.as_bytes());
        self.dev.console_flush();
    }

    // 添加命令到历史记录
    fn add_to_history(&mut self, line: &[u8]) {
        if line.is_empty() {
            return;
        }

        // 如果和最后一条记录相同，不添加
        if let Some(last) = self.history.last() {
            if last.as_slice() == line {
                return;
            }
        }

        let mut history_line = HeaplessVec::new();
        for &byte in line.iter().take(LINE_BUFFER_SIZE) {
            if history_line.push(byte).is_err() {
                break;
            }
        }

        if self.history.len() >= HISTORY_SIZE {
            // 移除最旧的记录
            self.history.remove(0);
        }

        let _ = self.history.push(history_line);
    }

    // 从历史记录加载
    fn load_from_history(&mut self, index: usize) {
        if index < self.history.len() {
            self.clear_current_line();
            self.current_line.clear();

            for &byte in self.history[index].iter() {
                let _ = self.current_line.push(byte);
            }

            self.cursor_pos = self.current_line.len();
            self.redraw_line();
        }
    }

    // 清除当前行显示
    fn clear_current_line(&mut self) {
        // 移动到行首
        self.dev.console_putc(b'\r');
        // 清除整行
        self.dev.console_write(b"\x1b[K");
        self.show_prompt();
    }

    // 重绘当前行
    fn redraw_line(&mut self) {
        self.dev.console_write(self.current_line.as_slice());
        // 移动光标到正确位置
        if self.cursor_pos < self.current_line.len() {
            let moves_back = self.current_line.len() - self.cursor_pos;
            for _ in 0..moves_back {
                self.dev.console_putc(b'\x08'); // 退格
            }
        }
        self.dev.console_flush();
    }

    // 向左移动光标
    fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.dev.console_putc(b'\x08'); // 退格
            self.dev.console_flush();
        }
    }

    // 向右移动光标
    fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.current_line.len() {
            self.dev.console_putc(self.current_line[self.cursor_pos]);
            self.cursor_pos += 1;
            self.dev.console_flush();
        }
    }

    // 在光标位置插入字符
    fn insert_char(&mut self, c: u8) {
        if self.current_line.len() >= LINE_BUFFER_SIZE {
            return;
        }

        // 如果光标在末尾，直接添加
        if self.cursor_pos == self.current_line.len() {
            if self.current_line.push(c).is_ok() {
                self.dev.console_putc(c);
                self.cursor_pos += 1;
                self.dev.console_flush();
            }
        } else {
            // 在中间插入字符 - 使用 Vec 的 insert 方法更简单
            let mut temp_vec = alloc::vec::Vec::new();

            // 复制光标前的字符
            for i in 0..self.cursor_pos {
                temp_vec.push(self.current_line[i]);
            }

            // 插入新字符
            temp_vec.push(c);

            // 复制光标后的字符
            for i in self.cursor_pos..self.current_line.len() {
                temp_vec.push(self.current_line[i]);
            }

            // 清空原来的 current_line 并重新填充
            self.current_line.clear();
            for &byte in temp_vec.iter().take(LINE_BUFFER_SIZE) {
                if self.current_line.push(byte).is_err() {
                    break;
                }
            }

            self.cursor_pos += 1;

            // 重绘整行
            self.clear_current_line();
            self.redraw_line();
        }
    }

    // 删除光标前的字符
    fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            // 移动后面的字符向前
            for i in self.cursor_pos..self.current_line.len() {
                if i > 0 {
                    let next_char = self.current_line[i];
                    self.current_line[i - 1] = next_char;
                }
            }

            if !self.current_line.is_empty() {
                self.current_line.pop();
            }
            self.cursor_pos -= 1;

            // 重绘整行
            self.clear_current_line();
            self.redraw_line();
        }
    }

    async fn _start<T>(&mut self, on_cmd: fn(Vec<String>) -> T)
    where
        T: core::future::Future<Output = ()> + 'static,
    {
        self.dev.console_flush_rx();
        crate::println!("Console started. Type 'help' for commands.");
        self.show_prompt();

        loop {
            let c = self.dev.console_getc();
            if let Some(b) = c {
                match self.escape_state {
                    EscapeState::Normal => {
                        match b {
                            // 回车或换行，处理命令
                            b'\r' | b'\n' => {
                                self.dev.console_putc(b'\r');
                                self.dev.console_putc(b'\n');
                                self.dev.console_flush();

                                if !self.current_line.is_empty() {
                                    // 创建一个临时的字节数组来避免借用检查问题
                                    let mut line_bytes = HeaplessVec::<u8, LINE_BUFFER_SIZE>::new();
                                    for &byte in self.current_line.iter() {
                                        let _ = line_bytes.push(byte);
                                    }

                                    let line_str = core::str::from_utf8(line_bytes.as_slice())
                                        .unwrap_or("")
                                        .trim();
                                    if !line_str.is_empty() {
                                        // 添加到历史记录
                                        self.add_to_history(line_str.as_bytes());
                                        let cmd_segs = line_str.split(";");
                                        for cmd_seg in cmd_segs {
                                            let args: Vec<String> = cmd_seg
                                                .split_whitespace()
                                                .map(|s| s.to_string())
                                                .collect();
                                            if args.len() > 0 {
                                                self.dev.console_flush();
                                                self.dev.console_flush_rx();
                                                self.signal_interrupt.clear();
                                                self.signal_handler =
                                                    SignalHandler::new(default_signal_handler);
                                                if let Some(cmd) = args.get(0) {
                                                    // let pid = Executor::spawn(cmd.clone(), Box::pin(on_cmd(args)));
                                                    let pid = Executor::spawn(
                                                        cmd.clone(),
                                                        Box::pin(async move {
                                                            let wait_signal_future = async {
                                                                loop {
                                                                    if let Some(sig) =
                                                                        Console::ref_mut()
                                                                            .signal_interrupt
                                                                            .pop()
                                                                    {
                                                                        Console::ref_mut()
                                                                            .signal_handler
                                                                            .call(sig)
                                                                            .await;
                                                                    }
                                                                    yield_now().await;
                                                                }
                                                            };
                                                            let cmd_future =
                                                                async move { on_cmd(args).await };
                                                            select! {
                                                                _ = wait_signal_future => {},
                                                                _ = cmd_future => {},
                                                            }
                                                        }),
                                                    );
                                                    Console::join(pid).await;
                                                }
                                            }
                                        }
                                    }
                                }

                                self.current_line.clear();
                                self.cursor_pos = 0;
                                self.history_index = None;
                                self.show_prompt();
                            }
                            // Ctrl+C (ASCII 3) - 终止当前输入
                            3 => {
                                self.dev.console_putc(b'^');
                                self.dev.console_putc(b'C');
                                self.dev.console_putc(b'\r');
                                self.dev.console_putc(b'\n');
                                self.dev.console_flush();
                                self.current_line.clear();
                                self.cursor_pos = 0;
                                self.history_index = None;
                                self.show_prompt();
                            }
                            // 退格键
                            8 | 127 => {
                                self.backspace();
                            }
                            // ESC序列开始
                            27 => {
                                self.escape_state = EscapeState::Escape;
                            }
                            // 可打印字符和空格
                            b if b.is_ascii_graphic() || b == b' ' => {
                                self.insert_char(b);
                            }
                            // 忽略其他控制字符
                            _ => {}
                        }
                    }
                    EscapeState::Escape => match b {
                        b'[' => {
                            self.escape_state = EscapeState::Bracket;
                        }
                        _ => {
                            self.escape_state = EscapeState::Normal;
                        }
                    },
                    EscapeState::Bracket => {
                        match b {
                            // 上箭头
                            b'A' => {
                                if !self.history.is_empty() {
                                    let new_index = match self.history_index {
                                        None => self.history.len() - 1,
                                        Some(idx) if idx > 0 => idx - 1,
                                        Some(_) => 0,
                                    };
                                    self.history_index = Some(new_index);
                                    self.load_from_history(new_index);
                                }
                            }
                            // 下箭头
                            b'B' => {
                                match self.history_index {
                                    Some(idx) if idx < self.history.len() - 1 => {
                                        self.history_index = Some(idx + 1);
                                        self.load_from_history(idx + 1);
                                    }
                                    Some(_) => {
                                        // 到达历史记录末尾，清空当前行
                                        self.history_index = None;
                                        self.clear_current_line();
                                        self.current_line.clear();
                                        self.cursor_pos = 0;
                                    }
                                    None => {}
                                }
                            }
                            // 右箭头
                            b'C' => {
                                self.move_cursor_right();
                            }
                            // 左箭头
                            b'D' => {
                                self.move_cursor_left();
                            }
                            _ => {}
                        }
                        self.escape_state = EscapeState::Normal;
                    }
                }
            }
            crate::sys::yield_now().await;
        }
    }

    pub fn start<T>(on_cmd: fn(Vec<String>) -> T, on_init: fn())
    where
        T: core::future::Future<Output = ()> + 'static,
    {
        Executor::spawn("console", Box::pin(Console::ref_mut()._start(on_cmd)));
        on_init();
        Executor::run();
    }

    pub async fn join(id: u16) {
        loop {
            if !Executor::is_running(id) {
                break;
            }
            yield_now().await;
        }
    }

    pub fn set_signal_handler<F, Fut>(handler: F)
    where
        F: Fn(u8) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let console = Console::ref_mut();
        console.signal_handler = SignalHandler::new(handler);
    }

    pub fn set_signal_handler_sync<F>(handler: F)
    where
        F: Fn(u8) + 'static,
    {
        let console = Console::ref_mut();
        console.signal_handler = SignalHandler::from_sync(handler);
    }

    pub fn signal_interrupt(sig: u8) {
        let mut console = Console::ref_mut();
        let _ = console.signal_interrupt.push(sig);
    }

    pub async fn read_key_async() -> Option<Key> {
        let console = Console::ref_mut();
        let io = &mut console.dev;
        let mut escape_state = EscapeState::Normal;
        loop {
            if let Some(b) = io.console_getc() {
                match escape_state {
                    EscapeState::Normal => match b {
                        b'\r' | b'\n' => return Some(Key::Enter),
                        3 => return Some(Key::CtrlC),
                        8 | 127 => return Some(Key::Backspace),
                        27 => {
                            escape_state = EscapeState::Escape;
                        }
                        b if b.is_ascii_graphic() || b == b' ' => {
                            return Some(Key::Char(b));
                        }
                        _ => {
                            return Some(Key::Unknown(b));
                        }
                    },
                    EscapeState::Escape => match b {
                        b'[' => {
                            escape_state = EscapeState::Bracket;
                        }
                        _ => {
                            escape_state = EscapeState::Normal;
                        }
                    },
                    EscapeState::Bracket => {
                        escape_state = EscapeState::Normal;
                        match b {
                            b'A' => return Some(Key::Up),
                            b'B' => return Some(Key::Down),
                            b'C' => return Some(Key::Right),
                            b'D' => return Some(Key::Left),
                            _ => return Some(Key::Unknown(b)),
                        }
                    }
                }
            }
            yield_now().await;
        }
    }
}

impl CmdParse for Console {
    async fn cmd_parse(args: Vec<String>) -> Option<Vec<String>> {
        if let Some(cmd) = args.get(0) {
            match cmd.as_str() {
                "help" | "?" => {
                    println!("Available commands:");
                    println!("  {:<40} {}", "help|?", "Show this help message");
                    println!("System commands:");
                    println!("  {:<40} {}", "reset", "Perform a system reset");
                    println!("  {:<40} {}", "date", "Get or set the system date and time");
                    println!("  {:<40} {}", "ps", "Show running tasks");
                    println!("  {:<40} {}", "kill", "Terminate a task");
                    println!("  {:<40} {}", "free", "Show free memory");
                    println!("  {:<40} {}", "pref", "Show task polling frequency");
                    println!("  {:<40} {}", "panic", "Trigger a panic");
                    return Some(args);
                }
                // "reset" => {
                //     Stm32::system_reset();
                //     return None;
                // }
                // "date" => {
                //     if let Some(arg1) = args.get(1) {
                //         // 设置日期时间
                //         if let Ok(dt) = NaiveDateTime::parse_from_str(arg1, "%Y-%m-%dT%H:%M:%S") {
                //             if Rtc::set_datetime(&dt) {
                //                 println!("Date and time set to: {}", dt);
                //             } else {
                //                 println!("Failed to set date and time.");
                //             }
                //         } else {
                //             println!("Invalid date format. Use 'YYYY-MM-DDTHH:MM:SS'");
                //         }
                //     } else {
                //         if let Some(dt) = Rtc::get_datetime() {
                //             println!("Current date and time: {}", dt);
                //         } else {
                //             println!("Failed to get current date, Maybe need setup rtc first.");
                //         }
                //     }
                //     return None;
                // }
                "ps" => {
                    let task_list = Executor::task_list();
                    println!("id\ttask");
                    for (task_id, name) in task_list.iter() {
                        println!("{}\t{}", task_id, name);
                    }
                    return None;
                }
                "kill" => {
                    if let Some(id_str) = args.get(1) {
                        if let Ok(id) = id_str.parse::<u16>() {
                            Executor::kill(id);
                            println!("Killed task with ID {}", id);
                        } else {
                            println!("Invalid task ID: {}", id_str);
                        }
                    } else {
                        println!("Usage: kill <task_id>");
                    }
                    return None;
                }
                "free" => {
                    let free_mem = calc_mem_free();
                    println!("Free memory: {} bytes", free_mem);
                    return None;
                }
                "pref" => {
                    let c = Rc::new(RefCell::new(0u32));
                    let f1 = async {
                        loop {
                            if let Ok(mut cc) = c.try_borrow_mut() {
                                *cc += 1;
                            }
                            yield_now().await;
                        }
                    };
                    let f2 = async {
                        loop {
                            sleep_ms(1000).await;
                            if let Ok(mut cc) = c.try_borrow_mut() {
                                println!("poll freq: {} times/sec", *cc);
                                *cc = 0;
                            }
                        }
                    };
                    join!(f1, f2);
                    return None;
                }
                "panic" => {
                    panic!("Manual panic triggered by user command");
                    // return None;
                }
                _ => return Some(args),
            }
        } else {
            return Some(args);
        }
    }
}

#[allow(unused)]
pub enum Key {
    Char(u8),
    CtrlC,
    Enter,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    Unknown(u8),
}

fn calc_mem_free() -> u32 {
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

    total
}
