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
            malloc(layout.size()) as *mut u8
        } else {
            // 对于较大的对齐要求，使用 aligned_alloc
            aligned_alloc(layout.align(), layout.size()) as *mut u8
        }

        // todo!("CAllocator alloc not implemented");
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr as *mut c_void);

        // todo!("CAllocator dealloc not implemented");
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

// #[global_allocator]
// pub static ALLOCATOR: CAllocator = CAllocator::new();
