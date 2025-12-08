use crate::driver::Driver;

pub struct LazyInit<T, F = fn() -> T>
where
    T: Driver,
    F: FnOnce() -> T,
{
    value: Option<T>,
    init_func: Option<F>,
}

impl<T, F> LazyInit<T, F>
where
    T: Driver,
    F: FnOnce() -> T,
{
    pub const fn new(f: F) -> Self {
        Self {
            value: None,
            init_func: Some(f),
        }
    }

    pub fn get_or_init(&mut self) -> &mut T {
        if self.value.is_none() {
            if let Some(f) = self.init_func.take() {
                self.value = Some(f());
                self.value.as_mut().unwrap().driver_init().unwrap();
            }
        }
        self.value.as_mut().unwrap()
    }

    pub fn init(&mut self) {
        let _ = self.get_or_init();
    }

    pub fn get(&mut self) -> Option<&mut T> {
        self.value.as_mut()
    }
}

impl<T, F> Drop for LazyInit<T, F>
where
    T: Driver,
    F: FnOnce() -> T,
{
    fn drop(&mut self) {
        if let Some(v) = &mut self.value {
            let _ = v.driver_deinit();
        }
    }
}

#[macro_export]
macro_rules! lazy_init {
    ($init:expr) => {
        $crate::driver::lazy_init::LazyInit::new(|| {$init} )
    };
}