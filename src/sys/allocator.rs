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

    // unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
    //     let ptr = self.alloc(layout);
    //     if !ptr.is_null() {
    //         core::ptr::write_bytes(ptr, 0, layout.size());
    //     }
    //     ptr
    // }

    // unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    //     let new_ptr = self.alloc(Layout::from_size_align_unchecked(new_size, layout.align()));
    //     if !new_ptr.is_null() && !ptr.is_null() {
    //         core::ptr::copy_nonoverlapping(ptr, new_ptr, layout.size().min(new_size));
    //         self.dealloc(ptr, layout);
    //     }
    //     new_ptr
    // }
}

#[global_allocator]
pub static ALLOCATOR: CAllocator = CAllocator::new();
