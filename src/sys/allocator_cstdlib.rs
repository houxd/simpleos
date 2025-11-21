use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_void;

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn aligned_alloc(alignment: usize, size: usize) -> *mut c_void;
}

pub struct CAllocator;

impl CAllocator {
    pub const fn new() -> Self {
        CAllocator
    }
}

unsafe impl GlobalAlloc for CAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= 8 {
            // 对于小对齐要求，直接使用 malloc
            unsafe { malloc(layout.size() as _) as *mut u8 }
        } else {
            // 对于较大的对齐要求，使用 aligned_alloc
            unsafe { aligned_alloc(layout.align() as _, layout.size() as _) as *mut u8 }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe {
            free(ptr as *mut ::core::ffi::c_void);
        }
    }
}

#[global_allocator]
pub static ALLOCATOR: CAllocator = CAllocator::new();
