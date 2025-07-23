use buddy_alloc::{BuddyAllocParam, FastAllocParam, NonThreadsafeAlloc};
use core::ptr::addr_of_mut;

const FAST_HEAP_SIZE: usize = 32 * 1024; // 32 KB
const HEAP_SIZE: usize = 1024 * 1024; // 1M
const LEAF_SIZE: usize = 16;

#[repr(align(64))]
struct Heap<const S: usize>([u8; S]);

static mut FAST_HEAP: Heap<FAST_HEAP_SIZE> = Heap([0u8; FAST_HEAP_SIZE]);
static mut HEAP: Heap<HEAP_SIZE> = Heap([0u8; HEAP_SIZE]);

#[cfg_attr(not(test), global_allocator)]
#[allow(unused)]
static ALLOC: NonThreadsafeAlloc = {
    let fast_param = FastAllocParam::new(addr_of_mut!(FAST_HEAP).cast(), FAST_HEAP_SIZE);
    let buddy_param = BuddyAllocParam::new(addr_of_mut!(HEAP).cast(), HEAP_SIZE, LEAF_SIZE);
    NonThreadsafeAlloc::new(fast_param, buddy_param)
};
