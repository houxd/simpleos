use core::future::Future;
use core::pin::Pin;
use core::cell::RefCell;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
extern crate alloc;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::collections::VecDeque;
use core::option::Option;

pub type BoxFuture = Pin<Box<dyn Future<Output = ()>>>;

pub struct Executor {
    tasks: Rc<RefCell<VecDeque<BoxFuture>>>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            tasks: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    pub fn spawn(&mut self, future: Pin<Box<dyn Future<Output = ()>>>) -> &mut Self {
        let task = Box::pin(future);
        self.tasks.borrow_mut().push_back(task);
        self
    }

    pub fn run(&mut self) {
        loop {
            let mut task = {
                let mut tasks = self.tasks.borrow_mut();
                if let Option::Some(task) = tasks.pop_front() {
                    task
                } else {
                    break; // 没有更多任务
                }
            };

            // 创建一个简单的waker
            let waker =  unsafe { Waker::from_raw(dummy_raw_waker()) };
            let mut context = Context::from_waker(&waker);

            // 轮询任务
            match task.as_mut().poll(&mut context) {
                Poll::Ready(()) => {
                    // 任务完成，什么都不做
                }
                Poll::Pending => {
                    // 任务未完成，重新加入队列
                    self.tasks.borrow_mut().push_back(task);
                }
            }
        }
    }

    /// 检查是否还有待执行的任务
    #[allow(unused)]
    pub fn has_tasks(&self) -> bool {
        !self.tasks.borrow().is_empty()
    }

    /// 获取当前任务队列长度
    #[allow(unused)]
    pub fn task_count(&self) -> usize {
        self.tasks.borrow().len()
    }
}

// 简单的 dummy waker
fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(core::ptr::null::<()>(), &VTABLE)
}
