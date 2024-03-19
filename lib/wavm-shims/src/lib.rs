//! Shim crate to mock Stylus's `vm_hooks`.
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ptr;
use std::slice;
use std::sync::Mutex;
use tiny_keccak::{Hasher, Keccak};

pub const WORD_BYTES: usize = 32;
pub type Bytes32 = [u8; WORD_BYTES];

pub static STORAGE: Lazy<Mutex<HashMap<Bytes32, Bytes32>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub unsafe fn read_bytes32(key: *const u8) -> Bytes32 {
    let mut res = Bytes32::default();
    ptr::copy(key, res.as_mut_ptr(), WORD_BYTES);
    res
}

pub unsafe fn write_bytes32(key: *mut u8, val: Bytes32) {
    ptr::copy(val.as_ptr(), key, WORD_BYTES);
}

#[no_mangle]
pub extern "C" fn storage_store_bytes32(key: *const u8, value: *const u8) {
    let (key, value) = unsafe { (read_bytes32(key), read_bytes32(value)) };

    STORAGE.lock().unwrap().insert(key, value);
}

#[no_mangle]
pub extern "C" fn storage_load_bytes32(key: *const u8, out: *mut u8) {
    let key = unsafe { read_bytes32(key) };

    let value = STORAGE
        .lock()
        .unwrap()
        .get(&key)
        .map(Bytes32::to_owned)
        .unwrap_or_default();

    unsafe { write_bytes32(out, value) };
}

const MSG_SENDER: &[u8; 42] = b"0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF";

#[no_mangle]
pub unsafe extern "C" fn msg_sender(sender: *mut u8) {
    let addr = const_hex::const_decode_to_array::<20>(MSG_SENDER).unwrap();
    std::ptr::copy(addr.as_ptr(), sender, 20);
}
