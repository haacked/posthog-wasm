#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[repr(C)]
pub struct HttpResponse {
    status: i32,
    body_ptr: *mut u8,
    body_len: usize,
}

#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut u8 {
    // Create a new vector with the specified size
    let mut buffer = Vec::with_capacity(size);
    // Ensure the vector has the exact size we want
    buffer.resize(size, 0);
    // Get the pointer to the vector's buffer
    let ptr = buffer.as_mut_ptr();
    // Prevent the vector from being deallocated
    std::mem::forget(buffer);
    // Return the pointer
    ptr
}

#[no_mangle]
pub extern "C" fn deallocate(ptr: *mut u8, size: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr, size, size);
        // Vector is dropped here, freeing the memory
    }
}

#[no_mangle]
pub extern "C" fn fetch_url(url_ptr: *const u8, url_len: usize) -> HttpResponse {
    // Convert the raw pointer to a string slice
    let url_bytes = unsafe { std::slice::from_raw_parts(url_ptr, url_len) };
    let url = std::str::from_utf8(url_bytes).unwrap_or("invalid url");
    
    println!("fetch_url called with: {}", url);

    let message = "Success".to_string();
    let body_vec = message.into_bytes();
    let body_len = body_vec.len();
    let body_ptr = Box::into_raw(body_vec.into_boxed_slice()) as *mut u8;
    HttpResponse { 
        status: 200, 
        body_ptr, 
        body_len 
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
