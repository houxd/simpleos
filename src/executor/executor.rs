use crate::executor::Runnable;
use crate::util::RingBuf;
use crate::{println, singleton, sys};
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use core::future::Future;
use core::option::Option;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub type TaskId = u16;
pub type TaskCountType = TaskId;
pub type ExitCode = i8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitStatus {
    Exited(ExitCode), // 任务正常退出，携带退出码
    ErrorPid,         // 等待了一个无效的PID
    NotExist,         // 任务不存在
    NotRunning,       // 当前任务未运行, 必须在任务上下文中调用
    Aborted,          // 任务异常中止, 不应该发生
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    SIGINT,     // 中断信号 (Ctrl+C)
    SIGTERM,    // 终止信号
    SIGKILL,    // 强制杀死(不可捕获)
    SIGSTOP,    // 暂停信号(不可捕获)
    SIGCONT,    // 继续执行
    SIGUSR(u8), // 用户自定义信号
    SIGNULL,    // 空信号, 用于占位
}

impl Default for Signal {
    fn default() -> Self {
        Signal::SIGNULL
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SignalAction {
    Ignore,              // 忽略信号
    Terminate(ExitCode), // 终止任务
    Pause,               // 暂停执行
    Continue,            // 继续执行
}

pub struct Task {
    id: TaskId,
    cmd: String,
    future: Pin<Box<dyn Future<Output = ExitCode>>>,
    paused: bool,                                                // 任务是否被暂停
    exited: Option<ExitCode>,                                    // 任务结束状态
    waiters: TaskCountType,                                      // 等待该任务完成的任务数量
    pending_signals: RingBuf<Signal, 4>,                         // 待处理的信号队列
    signal_handler: Option<Box<dyn Fn(Signal) -> SignalAction>>, // 信号处理器
}

impl Task {
    pub fn new(id: TaskId, cmd: String, future: Pin<Box<dyn Future<Output = ExitCode>>>) -> Self {
        Self {
            id,
            cmd,
            future,
            paused: false,
            exited: None,
            waiters: 0,
            pending_signals: RingBuf::new(),
            signal_handler: None,
        }
    }
}

pub struct Executor {
    tasks: VecDeque<Task>,
    next_id_hint: TaskId,
    current_task_id: Option<TaskId>,
}

singleton!(Executor {
    tasks: VecDeque::new(),
    next_id_hint: 0,
    current_task_id: None,
});

impl Executor {
    fn next_id(&mut self) -> TaskId {
        // 从提示位置开始搜索
        let mut candidate = self.next_id_hint;
        let start = candidate;

        loop {
            // 检查当前候选ID是否可用
            if !self.tasks.iter().any(|task| task.id == candidate) {
                self.next_id_hint = candidate.wrapping_add(1);
                return candidate;
            }

            // 尝试下一个ID
            candidate = candidate.wrapping_add(1);

            // 如果回到起始位置，说明所有ID都被占用（理论上不太可能）
            if candidate == start {
                panic!("No more available task IDs");
            }
        }
    }

    pub fn spawn(
        cmd: impl Into<String>,
        future: Pin<Box<dyn Future<Output = ExitCode>>>,
    ) -> TaskId {
        let executor = Executor::get_mut();
        let id = executor.next_id();
        executor.tasks.push_back(Task::new(id, cmd.into(), future));
        id
    }

    pub fn spawn_runnable(runner: Runnable, args: &[String]) -> TaskId {
        let executor = Executor::get_mut();
        let id = executor.next_id();
        executor
            .tasks
            .push_back(Task::new(id, runner.get_name(), runner.run(args)));
        id
    }

    pub fn default_signal_handler(signal: Signal) -> SignalAction {
        match signal {
            Signal::SIGINT | Signal::SIGTERM => SignalAction::Terminate(-1),
            Signal::SIGKILL => SignalAction::Terminate(-9),
            Signal::SIGSTOP => SignalAction::Pause,
            Signal::SIGCONT => SignalAction::Continue,
            Signal::SIGUSR(_) => SignalAction::Ignore,
            Signal::SIGNULL => SignalAction::Ignore,
        }
    }

    pub fn run() {
        let executor = Executor::get_mut();

        // 创建一个简单的waker
        let waker = unsafe { Waker::from_raw(dummy_raw_waker()) };
        let mut context = Context::from_waker(&waker);

        loop {
            let task: &mut Task = {
                if let Some(task) = executor.tasks.front_mut() {
                    task
                } else {
                    break; // 没有更多任务
                }
            };

            // 任务已退出
            if task.exited.is_some() {
                if task.waiters == 0 {
                    executor.tasks.pop_front(); // 任务无等待者，移除任务
                } else {
                    executor.tasks.rotate_left(1); // 继续下一个任务
                }
                continue;
            }

            // 设置当前任务ID, 用于 exit() 等函数使用
            executor.current_task_id = Some(task.id);

            // 处理待处理的信号
            while let Some(signal) = task.pending_signals.pop() {
                let action = if let Some(ref handler) = task.signal_handler {
                    match signal {
                        // SIGKILL 和 SIGSTOP 不能被捕获
                        Signal::SIGKILL => SignalAction::Terminate(-9),
                        Signal::SIGSTOP => SignalAction::Pause,
                        Signal::SIGNULL => SignalAction::Ignore,
                        _ => handler(signal),
                    }
                } else {
                    Self::default_signal_handler(signal)
                };

                match action {
                    SignalAction::Terminate(code) => {
                        task.exited = Some(code);
                        task.paused = false;
                        break;
                    }
                    SignalAction::Ignore => continue,
                    SignalAction::Pause => {
                        task.paused = true;
                        continue;
                    }
                    SignalAction::Continue => {
                        task.paused = false;
                        continue;
                    }
                }
            }

            // 如果任务被暂停，跳过执行
            if !task.paused {
                // 轮询任务
                match task.future.as_mut().poll(&mut context) {
                    Poll::Ready(exit_code) => {
                        task.exited = Some(exit_code); // 任务完成，设置退出码
                        if task.waiters == 0 {
                            // 任务无等待者，移除任务
                            executor.tasks.pop_front();
                        } else {
                            executor.tasks.rotate_left(1); // 继续下一个任务
                        }
                    }
                    Poll::Pending => {
                        executor.tasks.rotate_left(1); // 任务未完成，移到队列末尾
                    }
                }
            }

            // 清除当前任务
            let _ = executor.current_task_id.take().unwrap();
        }
    }

    /// 检查是否还有待执行的任务
    #[allow(unused)]
    pub fn has_tasks() -> bool {
        !Self::get_mut().tasks.is_empty()
    }

    /// 获取当前任务队列长度
    #[allow(unused)]
    pub fn task_count() -> usize {
        Self::get_mut().tasks.len()
    }

    /// 获取任务列表
    pub fn task_list() -> Vec<(TaskId, String)> {
        Self::get_mut()
            .tasks
            .iter()
            .map(|task| (task.id, task.cmd.clone()))
            .collect()
    }

    /// 获取当前运行任务ID
    pub fn current_task_id() -> Option<TaskId> {
        Self::get_mut().current_task_id
    }

    /// 杀死任务, 如果需要SIGKILL请使用 send_signal
    pub fn kill(id: TaskId) -> bool {
        Self::send_signal(id, Signal::SIGTERM)
    }

    /// 结束当前任务, 设置 exit 标志
    pub async fn exit(exit_code: ExitCode) -> ! {
        let current_id = match Self::current_task_id() {
            Some(id) => id,
            None => {
                panic!("BUG: exit() called outside of task context");
            }
        };

        // 在任务队列中找到当前任务，设置其 exited 标志
        if let Some(task) = Self::get_mut()
            .tasks
            .iter_mut()
            .find(|t| t.id == current_id)
        {
            task.exited = Some(exit_code);
        }

        // 进入循环后, yield会回到poll, 这个任务也就终止了
        sys::yield_now().await;

        // 理论上不应该到达这里
        panic!("BUG: exit() should not return");
    }

    /// 检查任务是否正在运行
    #[allow(unused)]
    pub fn is_running(id: TaskId) -> bool {
        // 检查是否为当前任务
        if Self::current_task_id() == Some(id) {
            return true;
        }
        // 检查任务队列中是否存在该任务
        let executor = Executor::get_mut();
        // let tasks = executor.tasks.borrow();
        executor.tasks.iter().any(|task| task.id == id)
    }

    /// 等待任务完成
    #[allow(unused)]
    pub async fn wait(id: TaskId) -> ExitStatus {
        // 获取当前任务id
        let current_id = match Self::current_task_id() {
            Some(id) => id,
            None => {
                // 当前没有任务在运行?
                return ExitStatus::NotRunning;
            }
        };

        // 不能等待自己
        if id == current_id {
            return ExitStatus::ErrorPid;
        }

        // 检查任务是否存在, 增加等待者计数
        if let Some(task) = Self::get_mut().tasks.iter_mut().find(|t| t.id == id) {
            task.waiters = task.waiters.wrapping_add(1);
        } else {
            return ExitStatus::NotExist;
        }

        // 轮询等待结果
        loop {
            // 让出当前任务，允许其他任务运行
            sys::yield_now().await;

            // 检查目标任务的状态
            if let Some(task) = Self::get_mut().tasks.iter_mut().find(|t| t.id == id) {
                if let Some(exit_code) = task.exited {
                    // 目标任务已退出，减少等待者计数
                    task.waiters = task.waiters.wrapping_sub(1);

                    // 任务已退出，返回结果
                    return ExitStatus::Exited(exit_code);
                }
            } else {
                // 任务不存在, 不应该发生
                return ExitStatus::Aborted;
            }
        }
    }

    /// 向任务发送信号
    pub fn send_signal(target_id: TaskId, signal: Signal) -> bool {
        if let Some(task) = Self::get_mut().tasks.iter_mut().find(|t| t.id == target_id) {
            task.pending_signals.push(signal);
            return true;
        }
        false
    }

    /// 注册信号处理器
    pub fn register_signal_handler<F>(handler: F)
    where
        F: Fn(Signal) -> SignalAction + 'static,
    {
        if let Some(current_id) = Self::current_task_id() {
            if let Some(task) = Self::get_mut()
                .tasks
                .iter_mut()
                .find(|t| t.id == current_id)
            {
                task.signal_handler = Some(Box::new(handler));
            }
        } else {
            println!("No current task");
        }
    }
}

// 简单的 dummy waker
fn dummy_raw_waker() -> RawWaker {
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }
    fn wake(_: *const ()) {}
    fn wake_by_ref(_: *const ()) {}
    fn drop(_: *const ()) {}
    const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
    RawWaker::new(core::ptr::null::<()>(), &VTABLE)
}
