use std::sync::Mutex;

use crate::engine::EditorEngine;
use crate::messages::EditorAction;

static ENGINE: Mutex<Option<EditorEngine>> = Mutex::new(None);
static RESPONSE_BUFFER: Mutex<Vec<u8>> = Mutex::new(Vec::new());

#[no_mangle]
pub extern "C" fn engine_init(width: f64, height: f64) {
    if let Ok(mut lock) = ENGINE.lock() {
        *lock = Some(EditorEngine::new(width, height));
    }
}

#[no_mangle]
pub extern "C" fn engine_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn engine_dealloc(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, len, len);
        }
    }
}

#[no_mangle]
pub extern "C" fn engine_dispatch(action_ptr: *const u8, action_len: usize) -> *const u8 {
    if action_ptr.is_null() || action_len == 0 {
        return std::ptr::null();
    }

    let action_slice = unsafe { std::slice::from_raw_parts(action_ptr, action_len) };
    let action_str = match std::str::from_utf8(action_slice) {
        Ok(s) => s,
        Err(_) => return std::ptr::null(),
    };

    let action: EditorAction = match serde_json::from_str(action_str) {
        Ok(a) => a,
        Err(_) => return std::ptr::null(),
    };

    let mut engine_lock = match ENGINE.lock() {
        Ok(l) => l,
        Err(_) => return std::ptr::null(),
    };

    let engine = match engine_lock.as_mut() {
        Some(e) => e,
        None => return std::ptr::null(),
    };

    let response = engine.dispatch(action);
    let json_bytes = match serde_json::to_vec(&response) {
        Ok(b) => b,
        Err(_) => return std::ptr::null(),
    };

    let mut buf_lock = match RESPONSE_BUFFER.lock() {
        Ok(l) => l,
        Err(_) => return std::ptr::null(),
    };

    *buf_lock = json_bytes;
    buf_lock.as_ptr()
}

#[no_mangle]
pub extern "C" fn engine_response_len() -> usize {
    RESPONSE_BUFFER.lock().map(|b| b.len()).unwrap_or(0)
}
