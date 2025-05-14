// src/lib.rs
#![no_std]
extern crate alloc;

use alloc::{vec::Vec, format};
use core::slice;
use core::str;
#[cfg(not(test))]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    core::mem::forget(buf); // Prevent it from being freed
    ptr
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, size: usize) {
    unsafe { let _ = Vec::from_raw_parts(ptr, size, size); }
}

#[no_mangle]
pub extern "C" fn greet(ptr: *mut u8, len: usize) -> *mut u8 {
    let input = unsafe { str::from_utf8_unchecked(slice::from_raw_parts(ptr, len)) };
    let result = format!("Hello, {}!", input);
    let output = result.into_bytes();
    let out_ptr = alloc(output.len());
    unsafe {
        core::ptr::copy_nonoverlapping(output.as_ptr(), out_ptr, output.len());
    }
    out_ptr
}

#[no_mangle]
pub extern "C" fn greet_len(ptr: *mut u8, len: usize) -> usize {
    let input = unsafe { str::from_utf8_unchecked(slice::from_raw_parts(ptr, len)) };
    let result = format!("Hello, {}!", input);
    result.len()
}

#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
