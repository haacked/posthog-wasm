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

extern "C" {
    fn http_request(url_ptr: *const u8, url_len: usize, 
                    method_ptr: *const u8, method_len: usize, 
                    body_ptr: *const u8, body_len: usize) -> *const u8;
    fn http_request_len() -> usize;
}

#[no_mangle]
pub extern "C" fn capture(event_name_ptr: *const u8, event_name_len: usize,
                           distinct_id_ptr: *const u8, distinct_id_len: usize,
                           api_key_ptr: *const u8, api_key_len: usize) -> *mut u8 {
    unsafe {
        // Convert input parameters to Rust strings
        let event_name_slice = slice::from_raw_parts(event_name_ptr, event_name_len);
        let event_name = str::from_utf8(event_name_slice).unwrap_or("unknown_event");

        let distinct_id_slice = slice::from_raw_parts(distinct_id_ptr, distinct_id_len);
        let distinct_id = str::from_utf8(distinct_id_slice).unwrap_or("unknown_distinct_id");
        
        let api_key_slice = slice::from_raw_parts(api_key_ptr, api_key_len);
        let api_key = str::from_utf8(api_key_slice).unwrap_or(""); // Default to empty if not valid UTF-8

        send_event(event_name, distinct_id, api_key)
    }
}

fn send_event(event_name: &str, distinct_id: &str, api_key: &str) -> *mut u8 {
    let method_str = "POST";
    let method_bytes = method_str.as_bytes();

    let url_str = "http://localhost:8000/capture/"; // Standard PostHog CE/Cloud endpoint
    let url_bytes = url_str.as_bytes();

    
    let body_str = format!(r#"{{
        "api_key": "{}",
        "event": "{}",
        "distinct_id": "{}",
        "properties": {{
            "$lib": "posthog-wasm",
            "$lib_version": "0.1.0",
            "$geoip_disabled": true
        }}
    }}"#, api_key, event_name, distinct_id);
    let body_bytes = body_str.as_bytes();

    unsafe {
    // Call host with http_request, passing the defined URL and constructed body
    let resp_ptr = http_request(url_bytes.as_ptr(), url_bytes.len(), 
                                method_bytes.as_ptr(), method_bytes.len(), 
                                body_bytes.as_ptr(), body_bytes.len());
                                
        let resp_len = http_request_len();

        let out_ptr = alloc(resp_len);
        core::ptr::copy_nonoverlapping(resp_ptr, out_ptr, resp_len);
        out_ptr
    }
}

#[no_mangle]
pub extern "C" fn request(url_ptr: *const u8, url_len: usize, 
                           method_ptr: *const u8, method_len: usize, 
                           body_ptr: *const u8, body_len: usize) -> *mut u8 {
    unsafe {
        // Call host
        let resp_ptr = http_request(url_ptr, url_len, method_ptr, method_len, body_ptr, body_len);
        let resp_len = http_request_len();

        // Allocate memory for the response and copy it
        let out_ptr = alloc(resp_len);
        core::ptr::copy_nonoverlapping(resp_ptr, out_ptr, resp_len);
        out_ptr
    }
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
