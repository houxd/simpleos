use crate::console::CmdParser;
use crate::executor::{Executor, ExitCode};
use crate::sys::SimpleOs;
use crate::{println, singleton, sys};
use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

const HISTORY_SIZE: usize = 10; // 历史记录最大条数
const LINE_BUFFER_SIZE: usize = 512; // 每行最大字符数

#[derive(Debug, Clone, Copy)]
enum EscapeState {
    Normal,
    Escape,
    Bracket,
}

pub struct Console {
    prompt: String,
    // 历史记录
    history: VecDeque<Vec<u8>>,
    history_index: Option<usize>,
    // 当前编辑状态
    current_line: Vec<u8>,
    cursor_pos: usize,
    // ANSI转义序列状态
    escape_state: EscapeState,
    cmds_parser_list: VecDeque<Box<dyn CmdParser>>,
}

singleton!(Console {
    prompt: String::from("> "),
    history: VecDeque::new(),
    history_index: None,
    current_line: Vec::new(),
    cursor_pos: 0,
    escape_state: EscapeState::Normal,
    cmds_parser_list: VecDeque::new(),
});

#[allow(unused)]
impl Console {
    pub fn add_commands(cmds: impl CmdParser + 'static) {
        let console = Console::get_mut();
        console.cmds_parser_list.push_back(Box::new(cmds));
    }

    pub fn set_prompt(prompt: &str) {
        let console = Console::get_mut();
        console.prompt = String::from(prompt);
    }

    fn show_prompt(&mut self) {
        SimpleOs::tty().tty_write(self.prompt.as_bytes());
        SimpleOs::tty().tty_flush();
    }

    // 添加命令到历史记录
    fn add_to_history(&mut self, line: &[u8]) {
        if line.is_empty() {
            return;
        }

        // 如果和最后一条记录相同，不添加
        if let Some(last) = self.history.back() {
            // 改为 back()
            if last.as_slice() == line {
                return;
            }
        }

        let history_line = line.to_vec();

        if self.history.len() >= HISTORY_SIZE {
            // 移除最旧的记录
            self.history.pop_front();
        }

        self.history.push_back(history_line);
    }

    // 从历史记录加载
    fn load_from_history(&mut self, index: usize) {
        if index < self.history.len() {
            self.clear_current_line();
            self.current_line.clear();

            if let Some(history_line) = self.history.iter().nth(index) {
                // 使用 iter().nth(index)
                self.current_line = history_line.clone(); // 直接克隆
            }

            self.cursor_pos = self.current_line.len();
            self.redraw_line();
        }
    }

    // 清除当前行显示
    fn clear_current_line(&mut self) {
        // 移动到行首
        SimpleOs::tty().tty_putc(b'\r');
        // 清除整行
        SimpleOs::tty().tty_write(b"\x1b[K");
        self.show_prompt();
    }

    // 重绘当前行
    fn redraw_line(&mut self) {
        SimpleOs::tty().tty_write(self.current_line.as_slice());
        // 移动光标到正确位置
        if self.cursor_pos < self.current_line.len() {
            let moves_back = self.current_line.len() - self.cursor_pos;
            for _ in 0..moves_back {
                SimpleOs::tty().tty_putc(b'\x08'); // 退格
            }
        }
        SimpleOs::tty().tty_flush();
    }

    // 向左移动光标
    fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            SimpleOs::tty().tty_putc(b'\x08'); // 退格
            SimpleOs::tty().tty_flush();
        }
    }

    // 向右移动光标
    fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.current_line.len() {
            SimpleOs::tty().tty_putc(self.current_line[self.cursor_pos]);
            self.cursor_pos += 1;
            SimpleOs::tty().tty_flush();
        }
    }

    // 在光标位置插入字符
    fn insert_char(&mut self, c: u8) {
        if self.current_line.len() >= LINE_BUFFER_SIZE {
            return;
        }

        // 如果光标在末尾，直接添加
        if self.cursor_pos == self.current_line.len() {
            self.current_line.push(c); // Vec::push 不会失败（除非内存不足）
            SimpleOs::tty().tty_putc(c);
            self.cursor_pos += 1;
            SimpleOs::tty().tty_flush();
        } else {
            // 使用 Vec 的 insert 方法
            self.current_line.insert(self.cursor_pos, c); // 直接使用 insert
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

    async fn exec_cmd(&mut self, args: Vec<String>) {
        SimpleOs::tty().tty_flush();
        SimpleOs::tty().tty_clear_rx();
        if let Some(cmd) = args.get(0) {
            // 显示帮助
            if cmd == "help" || cmd == "?" {
                println!("Available commands:");
                let parser_list = &self.cmds_parser_list;
                for parser in parser_list.iter() {
                    let helps = parser.help();
                    for (cmd, desc) in helps.iter() {
                        println!("  {:<40} - {}", cmd, desc);
                    }
                }
                return;
            }

            // 执行命令
            let pid = Executor::spawn(
                cmd.clone(),
                Box::pin(async move {
                    let parser_list = &Console::get_mut().cmds_parser_list;
                    for parser in parser_list.iter() {
                        let exit_code = parser.parse(&args).await;
                        if exit_code != 127 {
                            return exit_code;
                        }
                    }
                    println!("Unknown command: {}", args.join(" "));
                    127
                }),
            );

            // 等待前台任务结束, 监听 Ctrl+C 终止 
            loop {
                sys::yield_now().await;

                if !Executor::is_running(pid) {
                    break;
                }
                
                // 监听 Ctrl+C 以终止前台任务
                if  SimpleOs::tty().tty_get_break() {
                    Executor::kill(pid);
                }
            }
        }
    }

    async fn try_parse_cmdline(&mut self) {
        if self.current_line.is_empty() {
            return;
        }

        // 创建一个临时的字节数组来避免借用检查问题
        let mut line_bytes = self.current_line.clone();
        let line_str = core::str::from_utf8(line_bytes.as_slice())
            .unwrap_or("")
            .trim();
        if line_str.is_empty() {
            return;
        }

        // 添加到历史记录
        self.add_to_history(line_str.as_bytes());
        let cmd_list = line_str.split(";");
        for cmd in cmd_list {
            let args: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();
            if args.len() > 0 {
                self.exec_cmd(args).await;
            }
        }
    }

    pub async fn start() -> ExitCode {
        let console = Console::get_mut();
        console._start().await;
        0
    }

    async fn _start(&mut self) {
        SimpleOs::tty().tty_flush();
        SimpleOs::tty().tty_clear_rx();
        crate::println!("Console started. Type 'help' for commands.");
        self.show_prompt();

        loop {
            let c = SimpleOs::tty().tty_getc();
            if let Some(b) = c {
                match self.escape_state {
                    EscapeState::Normal => {
                        match b {
                            // 回车或换行，处理命令
                            b'\r' | b'\n' => {
                                SimpleOs::tty().tty_putc(b'\r');
                                SimpleOs::tty().tty_putc(b'\n');
                                SimpleOs::tty().tty_flush();

                                self.try_parse_cmdline().await;

                                self.current_line.clear();
                                self.cursor_pos = 0;
                                self.history_index = None;
                                self.show_prompt();
                            }
                            // Ctrl+C (ASCII 3) - 终止当前输入
                            3 => {
                                SimpleOs::tty().tty_putc(b'^');
                                SimpleOs::tty().tty_putc(b'C');
                                SimpleOs::tty().tty_putc(b'\r');
                                SimpleOs::tty().tty_putc(b'\n');
                                SimpleOs::tty().tty_flush();
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
            sys::yield_now().await;
        }
    }

    pub async fn getc() -> u8 {
        let console = Console::get_mut();
        loop {
            if let Some(b) = SimpleOs::tty().tty_getc() {
                return b;
            }
            sys::yield_now().await;
        }
    }

    pub async fn readline(buffer: &mut [u8]) -> usize {
        let console = Console::get_mut();
        let mut index = 0usize;
        loop {
            if let Some(b) = SimpleOs::tty().tty_getc() {
                if b == b'\r' || b == b'\n' {
                    break;
                }
                if index < buffer.len() {
                    buffer[index] = b;
                    index += 1;
                }
            }
            sys::yield_now().await;
        }
        index
    }

    // pub async fn join(id: u16) {
    //     loop {
    //         if !Executor::is_running(id) {
    //             break;
    //         }
    //         sys::yield_now().await;
    //     }
    // }

    // pub async fn read_key() -> Option<Key> {
    //     let console = Console::get_mut();
    //     let io = SimpleOs::console();
    //     let mut escape_state = EscapeState::Normal;
    //     loop {
    //         if let Some(b) = io.console_getc() {
    //             match escape_state {
    //                 EscapeState::Normal => match b {
    //                     b'\r' | b'\n' => return Some(Key::Enter),
    //                     3 => return Some(Key::CtrlC),
    //                     8 | 127 => return Some(Key::Backspace),
    //                     27 => {
    //                         escape_state = EscapeState::Escape;
    //                     }
    //                     b if b.is_ascii_graphic() || b == b' ' => {
    //                         return Some(Key::Char(b));
    //                     }
    //                     _ => {
    //                         return Some(Key::Unknown(b));
    //                     }
    //                 },
    //                 EscapeState::Escape => match b {
    //                     b'[' => {
    //                         escape_state = EscapeState::Bracket;
    //                     }
    //                     _ => {
    //                         escape_state = EscapeState::Normal;
    //                     }
    //                 },
    //                 EscapeState::Bracket => {
    //                     escape_state = EscapeState::Normal;
    //                     match b {
    //                         b'A' => return Some(Key::Up),
    //                         b'B' => return Some(Key::Down),
    //                         b'C' => return Some(Key::Right),
    //                         b'D' => return Some(Key::Left),
    //                         _ => return Some(Key::Unknown(b)),
    //                     }
    //                 }
    //             }
    //         }
    //         sys::yield_now();
    //     }
    // }
}

// #[allow(unused)]
// pub enum Key {
//     Char(u8),
//     CtrlC,
//     Enter,
//     Backspace,
//     Up,
//     Down,
//     Left,
//     Right,
//     Unknown(u8),
// }
