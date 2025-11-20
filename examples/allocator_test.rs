#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;
use simpleos::CAllocator;

#[global_allocator]
pub static ALLOCATOR: CAllocator = CAllocator::new();

#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    let mut v = Vec::new();
    for i in 0..100 {
        let x = alloc::boxed::Box::new(i);
        v.push(x);
    }
}

use core::ffi::c_void;

#[unsafe(no_mangle)]
pub extern "C" fn malloc(_: usize) -> *mut c_void {
    return core::ptr::null_mut();
}
#[unsafe(no_mangle)]
pub extern "C" fn free(_: *mut c_void) {}
#[unsafe(no_mangle)]
pub extern "C" fn aligned_alloc(_: usize, _: usize) -> *mut c_void {
    return core::ptr::null_mut();
}
