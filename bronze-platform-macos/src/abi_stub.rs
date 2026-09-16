//! In-crate C ABI stand-in until story 2.3 links BronzeNative into Tauri.
//!
//! Enabled by the default `abi-stub` feature so `cargo test --workspace`
//! and clippy resolve symbols without a Swift static library. The stub
//! implements the same contract as BronzeNative.h (version 1, ptr+len
//! UTF-8, no strlen). Real Swift symbols replace this when `abi-stub`
//! is turned off.

use crate::abi::{
    BronzeNativeUtf8View, BRONZE_ABI_VERSION, BRONZE_STATUS_INVALID_UTF8, BRONZE_STATUS_OK,
};

#[no_mangle]
pub extern "C" fn bronze_native_abi_version() -> u32 {
    BRONZE_ABI_VERSION
}

#[no_mangle]
pub extern "C" fn bronze_native_validate_utf8(view: BronzeNativeUtf8View) -> u32 {
    if view.ptr.is_null() {
        return if view.len == 0 {
            BRONZE_STATUS_OK
        } else {
            BRONZE_STATUS_INVALID_UTF8
        };
    }
    let Some(len) = usize::try_from(view.len).ok() else {
        return BRONZE_STATUS_INVALID_UTF8;
    };
    // SAFETY: caller owns [ptr, ptr+len); null+nonzero is rejected above.
    let bytes = unsafe { std::slice::from_raw_parts(view.ptr, len) };
    if std::str::from_utf8(bytes).is_ok() {
        BRONZE_STATUS_OK
    } else {
        BRONZE_STATUS_INVALID_UTF8
    }
}

#[no_mangle]
pub extern "C" fn bronze_native_test_view_len(view: BronzeNativeUtf8View) -> u64 {
    view.len
}

#[no_mangle]
pub extern "C" fn bronze_native_init() {}

#[no_mangle]
pub extern "C" fn bronze_native_shutdown() {}
