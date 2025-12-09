use crate::{singleton, sys};
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::future::Future;
use core::option::Option;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub struct Task {
    id: u16,
    cmd: String,
    future: Pin<Box<dyn Future<Output = ()>>>,
}
impl Task {
    pub fn new(id: u16, cmd: String, future: Pin<Box<dyn Future<Output = ()>>>) -> Self {
        Task { id, cmd, future }
    }
}

pub struct Executor {
    tasks: Rc<RefCell<VecDeque<Task>>>,
    current_task_id: Option<u16>,
    exit: bool,
    next_id_hint: RefCell<u16>,
}

singleton!(Executor {
    tasks: Rc::new(RefCell::new(VecDeque::new())),
    current_task_id: None,
    next_id_hint: RefCell::new(0),
    exit: false,
});

impl Executor {
    fn next_id(&self) -> u16 {
        let tasks = self.tasks.borrow();
        let mut hint = self.next_id_hint.borrow_mut();

        // 从提示位置开始搜索
        let mut candidate = *hint;
        let start = candidate;

        loop {
            // 检查当前候选ID是否可用
            if !tasks.iter().any(|task| task.id == candidate) {
                *hint = if candidate == u16::MAX {
                    0
                } else {
                    candidate + 1
                };
                return candidate;
            }

            // 尝试下一个ID
            candidate = if candidate == u16::MAX {
                0
            } else {
                candidate + 1
            };

            // 如果回到起始位置，说明所有ID都被占用（理论上不太可能）
            if candidate == start {
                panic!("No more available task IDs");
            }
        }
    }

    pub fn spawn(cmd: impl Into<String>, future: Pin<Box<dyn Future<Output = ()>>>) -> u16 {
        let executor = Executor::get_mut();
        let id = executor.next_id();
        executor
            .tasks
            .borrow_mut()
            .push_back(Task::new(id, cmd.into(), future));
        id
    }

    pub fn run() {
        let executor = Executor::get_mut();
        loop {
            let mut task = {
                let mut tasks = executor.tasks.borrow_mut();
                if let Option::Some(task) = tasks.pop_front() {
                    task
                } else {
                    break; // 没有更多任务
                }
            };

            // 创建一个简单的waker
            let waker = unsafe { Waker::from_raw(dummy_raw_waker()) };
            let mut context = Context::from_waker(&waker);

            // 设置当前任务
            executor.current_task_id.replace(task.id);

            // 轮询任务
            match task.future.as_mut().poll(&mut context) {
                Poll::Ready(()) => {
                    // 任务完成，什么都不做
                }
                Poll::Pending => {
                    if executor.exit {
                        executor.exit = false;
                        // 任务被标记为退出，丢弃任务
                    } else {
                        // 任务未完成，重新加入队列
                        executor.tasks.borrow_mut().push_back(task);
                    }
                }
            }

            // 清除当前任务
            executor.current_task_id.take();
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
    pub fn task_list() -> Vec<(u16, String)> {
        Self::get_mut()
            .tasks
            .borrow()
            .iter()
            .map(|task| (task.id, task.cmd.clone()))
            .collect()
    }

    /// 获取当前运行任务ID
    pub fn current_task_id() -> Option<u16> {
        Self::get_mut().current_task_id.clone()
    }

    /// 杀死任务
    pub fn kill(id: u16) {
        // 如果是当前任务，则不执行任何操作
        if Self::current_task_id() == Some(id) {
            return;
        }
        // 从任务队列中移除该任务
        Self::get_mut()
            .tasks
            .borrow_mut()
            .retain(|task| task.id != id);
    }

    /// 结束当前任务, 设置 exit 标志
    pub async fn exit() {
        if Self::current_task_id().is_none() {
            panic!("exit() called outside of task context");
        }
        Self::get_mut().exit = true;
        // 结束当前poll, 这会终止这个任务的执行
        loop {
            sys::yield_now().await;
        }
    }

    /// 检查任务是否正在运行
    #[allow(unused)]
    pub fn is_running(id: u16) -> bool {
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
    pub async fn join(id: u16) {
        if Self::current_task_id() == Some(id) {
            return; // 不能等待自己
        }
        loop {
            {
                let executor = Executor::get_mut();
                let tasks = executor.tasks.borrow();
                if !tasks.iter().any(|task| task.id == id) {
                    break; // 任务已完成
                }
            }
            // 暂停当前任务，允许其他任务运行
            sys::yield_now().await;
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
