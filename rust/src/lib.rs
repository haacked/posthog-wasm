#![no_std]
extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::panic::PanicInfo;
use core::{slice, str};

// Serde imports
use serde_json::{Map, Value};

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[no_mangle]
pub extern "C" fn alloc_buffer(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    core::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn dealloc_buffer(ptr: *mut u8, size: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr, size, size);
    }
}

extern "C" {
    fn http_request(
        url_ptr: *const u8,
        url_len: usize,
        method_ptr: *const u8,
        method_len: usize,
        body_ptr: *const u8,
        body_len: usize,
    ) -> *const u8;
    fn http_request_len() -> usize;
    fn log_message(message: *const u8, message_len: usize);
}

// Helper function to merge two serde_json::Value instances.
// `base` provides the initial set of properties.
// `overrides` provides properties that should be added or will overwrite those in `base`.
// The function always aims to return a Value::Object.
fn merge_json_values(base: &Value, overrides: &Value) -> Value {
    let mut result_map = Map::new();

    // 1. Populate with base properties if base is an object
    if let Value::Object(base_map) = base {
        for (key, value) in base_map.iter() {
            result_map.insert(key.clone(), value.clone());
        }
    }

    // 2. Overlay with override properties if overrides is an object
    if let Value::Object(override_map) = overrides {
        for (key, value) in override_map.iter() {
            result_map.insert(key.clone(), value.clone()); // Overwrites if key exists
        }
    }

    Value::Object(result_map)
}

#[no_mangle]
pub extern "C" fn capture(
    event_name_ptr: *const u8,
    event_name_len: usize,
    distinct_id_ptr: *const u8,
    distinct_id_len: usize,
    api_key_ptr: *const u8,
    api_key_len: usize,
    properties_ptr: *const u8,
    properties_len: usize,
) -> *mut u8 {
    unsafe {
        let event_name_slice = slice::from_raw_parts(event_name_ptr, event_name_len);
        let event_name = str::from_utf8(event_name_slice).unwrap_or("unknown_event");

        let distinct_id_slice = slice::from_raw_parts(distinct_id_ptr, distinct_id_len);
        let distinct_id = str::from_utf8(distinct_id_slice).unwrap_or("unknown_distinct_id");

        let api_key_slice = slice::from_raw_parts(api_key_ptr, api_key_len);
        let api_key = str::from_utf8(api_key_slice).unwrap_or("");

        let properties_slice = slice::from_raw_parts(properties_ptr, properties_len);
        let properties_str = str::from_utf8(properties_slice).unwrap_or("{}");

        log(properties_str);

        let user_provided_properties: Value =
            serde_json::from_str(properties_str).unwrap_or_else(|_| Value::Object(Map::new())); // Default to empty object

        send_event(event_name, distinct_id, api_key, &user_provided_properties)
    }
}

fn send_event(
    event_name: &str,
    distinct_id: &str,
    api_key: &str,
    user_provided_properties: &Value,
) -> *mut u8 {
    let method_str = "POST";
    let method_bytes = method_str.as_bytes();

    let url_str = "/capture";
    let url_bytes = url_str.as_bytes();

    // Define default PostHog properties as a serde_json::Value
    let default_properties = serde_json::json!({
        "$lib": "posthog-wasm",
        "$lib_version": "0.1.0",
        "$geoip_disabled": true
    });

    // Merge default and user-provided properties using the helper function
    let final_properties = merge_json_values(&default_properties, user_provided_properties);

    // Construct the final event payload
    let event_payload = serde_json::json!({
        "api_key": api_key,
        "event": event_name,
        "distinct_id": distinct_id,
        "properties": final_properties // This will be a Value::Object
    });

    let body_str = serde_json::to_string(&event_payload)
        .unwrap_or_else(|_| String::from("{\"error\":\"Failed to serialize event payload\"}"));
    let body_bytes = body_str.as_bytes();

    unsafe {
        let resp_ptr = http_request(
            url_bytes.as_ptr(),
            url_bytes.len(),
            method_bytes.as_ptr(),
            method_bytes.len(),
            body_bytes.as_ptr(),
            body_bytes.len(),
        );
        let resp_len = http_request_len();
        let out_ptr = alloc_buffer(resp_len);
        core::ptr::copy_nonoverlapping(resp_ptr, out_ptr, resp_len);
        out_ptr
    }
}

fn log(message: &str) {
    unsafe {
        log_message(message.as_ptr(), message.len());
    }
}

#[no_mangle]
pub extern "C" fn request(
    url_ptr: *const u8,
    url_len: usize,
    method_ptr: *const u8,
    method_len: usize,
    body_ptr: *const u8,
    body_len: usize,
) -> *mut u8 {
    unsafe {
        // Call host
        let resp_ptr = http_request(url_ptr, url_len, method_ptr, method_len, body_ptr, body_len);
        let resp_len = http_request_len();

        // Allocate memory for the response and copy it
        let out_ptr = alloc_buffer(resp_len);
        core::ptr::copy_nonoverlapping(resp_ptr, out_ptr, resp_len);
        out_ptr
    }
}
