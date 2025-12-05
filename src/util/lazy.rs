pub struct Lazy<T, F = fn() -> T>
where
    F: FnOnce() -> T,
{
    value: Option<T>,
    init_func: Option<F>,
}

impl<T, F> Lazy<T, F>
where
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
            }
        }
        self.value.as_mut().unwrap()
    }
}
