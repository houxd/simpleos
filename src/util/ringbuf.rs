use core::mem::MaybeUninit;

/// 环形缓冲区结构体，支持泛型T和常量大小N（必须实现Copy和Default）。
pub struct RingBuf<T, const N: usize> {
    buf: [T; N],
    head: usize, // 写指针
    tail: usize, // 读指针
}

/// 环形缓冲区的迭代器结构体。
pub struct RingBufIter<'a, T, const N: usize> {
    buf: &'a [T; N],
    current: usize,
    remaining: usize,
}

impl<'a, T: Copy + Default, const N: usize> Iterator for RingBufIter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let item = &self.buf[self.current];
        self.current = (self.current + 1) % N;
        self.remaining -= 1;
        Some(item)
    }
}

impl<'a, T: Copy + Default, const N: usize> IntoIterator for &'a RingBuf<T, N> {
    type Item = &'a T;
    type IntoIter = RingBufIter<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        RingBufIter {
            buf: &self.buf,
            current: self.tail,
            remaining: self.len(),
        }
    }
}

impl<T: Copy + Default, const N: usize> RingBuf<T, N> {
    /// 创建新缓冲区（N必须大于1）。
    pub const fn new() -> Self {
        assert!(N > 1, "RINGBUF SIZE ERR");
        Self {
            buf: unsafe {
                MaybeUninit::<[T; N]>::zeroed().assume_init()
            },
            head: 0,
            tail: 0,
        }
    }

    /// 返回缓冲区容量（实际可用大小）。
    pub const fn capacity(&self) -> usize {
        N - 1
    }

    /// 返回当前元素数量。
    pub fn len(&self) -> usize {
        let h = self.head;
        let t = self.tail;
        if h >= t { h - t } else { N - t + h }
    }

    /// 检查是否为空。
    pub fn is_empty(&self) -> bool {
        self.tail == self.head
    }

    /// 检查是否满。
    pub fn is_full(&self) -> bool {
        (self.head + 1) % N == self.tail
    }

    /// 清空缓冲区。
    pub fn clear(&mut self) {
        self.tail = self.head;
    }

    /// 返回队头元素的引用（如果非空）。
    pub fn front(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self.buf[self.tail])
        }
    }

    /// 返回队尾元素的引用（如果非空）。
    pub fn tail(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            let idx = if self.head == 0 { N - 1 } else { self.head - 1 };
            Some(&self.buf[idx])
        }
    }

    /// 移除队头元素。
    pub fn pop(&mut self) -> Option<T> {
        if !self.is_empty() {
            let value = self.buf[self.tail];
            self.tail = (self.tail + 1) % N;
            Some(value)
        } else {
            None
        }
    }

    /// 添加一个元素，返回是否成功（失败表示满）。
    pub fn push(&mut self, value: T) -> bool {
        let next = (self.head + 1) % N;
        if next == self.tail {
            false
        } else {
            self.buf[self.head] = value;
            self.head = next;
            true
        }
    }

    /// 检查数据是否连续（适合高效拷贝）。
    pub fn is_continuous(&self) -> bool {
        self.is_empty() || self.tail < self.head
    }

    /// 拷贝最多n个元素到dest切片，返回实际拷贝数量。
    pub fn copy_to(&self, dest: &mut [T]) -> usize {
        let n = dest.len().min(self.len());
        if n == 0 {
            return 0;
        }
        if self.is_continuous() {
            // 一段连续
            dest[..n].copy_from_slice(&self.buf[self.tail..self.tail + n]);
        } else {
            // 分两段
            let first = N - self.tail;
            if first >= n {
                dest[..n].copy_from_slice(&self.buf[self.tail..self.tail + n]);
            } else {
                dest[..first].copy_from_slice(&self.buf[self.tail..]);
                dest[first..n].copy_from_slice(&self.buf[..n - first]);
            }
        }
        n
    }
}


