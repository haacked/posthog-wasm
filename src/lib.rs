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
pub extern "C" fn fetch_url(url_ptr: *const u8, url_len: usize) -> HttpResponse {
    println!("fetch_url url_ptr: {:?}", url_ptr);
    println!("fetch_url url_len: {:?}", url_len);

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
