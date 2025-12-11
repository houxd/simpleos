use crate::executor::Runner;
use crate::{singleton, sys};
use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::future::Future;
use core::option::Option;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub type TaskId = u16;

pub type ExitCode = i8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitStatus {
    Exited(ExitCode), // 任务正常退出，携带退出码
    Killed,           // 任务被杀死
    Aborted,          // 任务异常中止
    ErrorPid,         // 等待了一个无效的PID
    NotExist,         // 任务不存在
    NoRunning,        // 当前没有任务在运行
}

struct Waiter {
    waiter_id: TaskId,          // 等待的任务ID
    result: Option<ExitStatus>, // 存储结果
}

pub struct Task {
    id: TaskId,
    cmd: String,
    future: Pin<Box<dyn Future<Output = ExitCode>>>,
    exit_request: Option<ExitCode>,
}

pub struct Executor {
    tasks: RefCell<VecDeque<Task>>,
    next_id_hint: RefCell<TaskId>,
    current_task_id: RefCell<Option<TaskId>>,
    waiters: RefCell<BTreeMap<TaskId, Vec<Waiter>>>,
}

singleton!(Executor {
    tasks: RefCell::new(VecDeque::new()),
    next_id_hint: RefCell::new(0),
    current_task_id: RefCell::new(None),
    waiters: RefCell::new(BTreeMap::new()),
});

impl Executor {
    fn next_id(&self) -> TaskId {
        let tasks = self.tasks.borrow();
        let mut hint = self.next_id_hint.borrow_mut();

        // 从提示位置开始搜索
        let mut candidate = *hint;
        let start = candidate;

        loop {
            // 检查当前候选ID是否可用
            if !tasks.iter().any(|task| task.id == candidate) {
                *hint = candidate.wrapping_add(1);
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
        executor.tasks.borrow_mut().push_back(Task {
            id,
            cmd: cmd.into(),
            future,
            exit_request: None,
        });
        id
    }

    pub fn spawn_runner(runner: Runner, args: &[String]) -> TaskId {
        let executor = Executor::get_mut();
        let id = executor.next_id();
        executor.tasks.borrow_mut().push_back(Task {
            id,
            cmd: runner.get_name(),
            future: runner.run(args),
            exit_request: None,
        });
        id
    }

    pub fn run() {
        let executor = Executor::get_mut();
        loop {
            let mut task = {
                let mut tasks = executor.tasks.borrow_mut();
                if let Some(task) = tasks.pop_front() {
                    task
                } else {
                    break; // 没有更多任务
                }
            };

            // 创建一个简单的waker
            let waker = unsafe { Waker::from_raw(dummy_raw_waker()) };
            let mut context = Context::from_waker(&waker);

            // 设置当前任务
            *executor.current_task_id.borrow_mut() = Some(task.id);

            // 轮询任务
            let exit_status = match task.future.as_mut().poll(&mut context) {
                Poll::Ready(exit_code) => Some(ExitStatus::Exited(exit_code)),
                Poll::Pending => {
                    // 检查是否有退出请求
                    if let Some(exit_code) = task.exit_request.take() {
                        Some(ExitStatus::Exited(exit_code))
                    } else {
                        // 任务未完成，重新加入队列
                        executor.tasks.borrow_mut().push_back(task);
                        None
                    }
                }
            };

            // 清除当前任务
            let task_id = executor.current_task_id.borrow_mut().take().unwrap();

            // 如果任务完成，通知等待者
            if let Some(exit_status) = exit_status {
                let mut waiters = executor.waiters.borrow_mut();
                if let Some(waiter_list) = waiters.get_mut(&task_id) {
                    // 填充所有等待者的结果
                    for waiter in waiter_list.iter_mut() {
                        waiter.result = Some(exit_status);
                    }
                }
                // 注意：不删除等待者列表，让wait()函数自己清理
            }
        }
    }

    /// 检查是否还有待执行的任务
    #[allow(unused)]
    pub fn has_tasks() -> bool {
        !Self::get_mut().tasks.borrow().is_empty()
    }

    /// 获取当前任务队列长度
    #[allow(unused)]
    pub fn task_count() -> usize {
        Self::get_mut().tasks.borrow().len()
    }

    /// 获取任务列表
    pub fn task_list() -> Vec<(TaskId, String)> {
        Self::get_mut()
            .tasks
            .borrow()
            .iter()
            .map(|task| (task.id, task.cmd.clone()))
            .collect()
    }

    /// 获取当前运行任务ID
    pub fn current_task_id() -> Option<TaskId> {
        *Self::get_mut().current_task_id.borrow()
    }

    /// 杀死任务
    pub fn kill(id: TaskId) -> bool {
        if Self::current_task_id() == Some(id) {
            return false; // 不能杀死自己
        }

        let existed = {
            let mut tasks = Self::get_mut().tasks.borrow_mut();
            let before = tasks.len();
            tasks.retain(|task| task.id != id);
            tasks.len() < before
        };

        if existed {
            let mut waiters = Self::get_mut().waiters.borrow_mut();

            // 通知等待被杀死任务的等待者
            if let Some(waiter_list) = waiters.get_mut(&id) {
                for waiter in waiter_list.iter_mut() {
                    waiter.result = Some(ExitStatus::Killed);
                }
            }

            // 清理该任务作为等待者的记录
            for (_, waiter_list) in waiters.iter_mut() {
                waiter_list.retain(|w| w.waiter_id != id);
            }

            // 清理空的等待者列表
            waiters.retain(|_, waiter_list| !waiter_list.is_empty());
        }

        existed
    }

    /// 结束当前任务, 设置 exit 标志
    pub async fn exit(exit_code: ExitCode) -> ! {
        let current_id = match Self::current_task_id() {
            Some(id) => id,
            None => {
                panic!("BUG: exit() called outside of task context");
            }
        };

        // 在任务队列中找到当前任务，设置其 exit_request
        {
            let mut tasks = Self::get_mut().tasks.borrow_mut();
            if let Some(task) = tasks.iter_mut().find(|t| t.id == current_id) {
                task.exit_request = Some(exit_code);
            }
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
        let tasks = executor.tasks.borrow();
        tasks.iter().any(|task| task.id == id)
    }

    /// 等待任务完成
    #[allow(unused)]
    pub async fn wait(id: TaskId) -> ExitStatus {
        // 获取当前任务id
        let current_id = match Self::current_task_id() {
            Some(id) => id,
            None => {
                // 当前没有任务在运行?
                return ExitStatus::NoRunning;
            }
        };

        // 不能等待自己
        if id == current_id {
            return ExitStatus::ErrorPid;
        }

        // 用代码块限制借用范围
        let task_exists = {
            let tasks = Self::get_mut().tasks.borrow();
            tasks.iter().any(|task| task.id == id)
        }; // tasks 在这里释放

        if !task_exists {
            return ExitStatus::NotExist;
        }

        // 注册为等待者
        {
            let mut waiters = Self::get_mut().waiters.borrow_mut();
            let waiter_list = waiters.entry(id).or_insert_with(Vec::new);
            waiter_list.push(Waiter {
                waiter_id: current_id,
                result: None,
            });
        }

        // 轮询等待结果
        loop {
            // 让出当前任务，允许其他任务运行
            sys::yield_now().await;

            // 检查等待者是否仍然存在
            let waiters = Self::get_mut().waiters.borrow();
            if let Some(waiter_list) = waiters.get(&id) {
                for w in waiter_list.iter() {
                    if w.waiter_id == current_id {
                        if let Some(exit_status) = w.result {
                            // 清理等待者
                            drop(waiters); // 释放不可变借用
                            let mut waiters = Self::get_mut().waiters.borrow_mut();
                            if let Some(waiter_list) = waiters.get_mut(&id) {
                                waiter_list.retain(|w| w.waiter_id != current_id);
                                // 如果没有等待者了，删除整个条目
                                if waiter_list.is_empty() {
                                    waiters.remove(&id);
                                }
                            }
                            // 结果已就绪
                            return exit_status;
                        }
                    }
                }
            } else {
                // 已被清理，任务不存在
                return ExitStatus::Aborted;
            }
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
