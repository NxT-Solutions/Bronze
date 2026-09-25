use crate::abi::{
    BronzeNativeUtf8View, BRONZE_PASTEBOARD_KIND_HTML, BRONZE_PASTEBOARD_KIND_NONE,
    BRONZE_PASTEBOARD_KIND_PLAIN, BRONZE_PASTEBOARD_KIND_RTF, BRONZE_STATUS_OK,
};
use crate::bridge::NativeError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClipboardTextKind {
    Html,
    Rtf,
    Plain,
}

#[derive(Clone, Eq, PartialEq)]
pub struct BoundedClipboardRead {
    pub kind: ClipboardTextKind,
    pub payload: String,
    pub post_copy_generation: u64,
    pub snapshot: Vec<u8>,
}

impl std::fmt::Debug for BoundedClipboardRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundedClipboardRead")
            .field("kind", &self.kind)
            .field("len", &self.payload.len())
            .field("post_copy_generation", &self.post_copy_generation)
            .field("snapshot_len", &self.snapshot.len())
            .finish()
    }
}

pub fn native_pasteboard_write(plain: &str, html: &str) -> Result<(), NativeError> {
    let plain_view = BronzeNativeUtf8View {
        ptr: plain.as_ptr(),
        len: plain.len() as u64,
    };
    let html_view = BronzeNativeUtf8View {
        ptr: html.as_ptr(),
        len: html.len() as u64,
    };
    let status = unsafe { crate::abi::bronze_native_pasteboard_write(plain_view, html_view) };
    crate::bridge::map_status(status)
}

pub fn native_bounded_copy_read(target_pid: i32) -> Result<BoundedClipboardRead, NativeError> {
    let mut kind = 0u32;
    let mut post_copy = 0u64;
    let mut payload = BronzeNativeUtf8View {
        ptr: std::ptr::null(),
        len: 0,
    };
    let mut snapshot = BronzeNativeUtf8View {
        ptr: std::ptr::null(),
        len: 0,
    };
    let status = unsafe {
        crate::abi::bronze_native_bounded_copy_read(
            target_pid,
            &mut kind,
            &mut post_copy,
            &mut payload,
            &mut snapshot,
        )
    };
    if status != BRONZE_STATUS_OK {
        if !payload.ptr.is_null() {
            free_view(payload);
        }
        if !snapshot.ptr.is_null() {
            free_view(snapshot);
        }
        return Err(match crate::bridge::map_status(status) {
            Ok(()) => NativeError::Degraded,
            Err(err) => err,
        });
    }
    if kind == BRONZE_PASTEBOARD_KIND_NONE {
        free_view(payload);
        free_view(snapshot);
        return Err(NativeError::Degraded);
    }
    let kind = match kind {
        BRONZE_PASTEBOARD_KIND_HTML => ClipboardTextKind::Html,
        BRONZE_PASTEBOARD_KIND_RTF => ClipboardTextKind::Rtf,
        BRONZE_PASTEBOARD_KIND_PLAIN => ClipboardTextKind::Plain,
        _ => {
            free_view(payload);
            free_view(snapshot);
            return Err(NativeError::Degraded);
        }
    };
    let text = take_utf8(payload).ok_or(NativeError::InvalidUtf8)?;
    let snap = take_bytes(snapshot);
    Ok(BoundedClipboardRead {
        kind,
        payload: text,
        post_copy_generation: post_copy,
        snapshot: snap,
    })
}

pub fn native_pasteboard_restore_if_unchanged(
    snapshot: &[u8],
    expected_count: u64,
) -> Result<(), NativeError> {
    let view = BronzeNativeUtf8View {
        ptr: snapshot.as_ptr(),
        len: snapshot.len() as u64,
    };
    let status =
        unsafe { crate::abi::bronze_native_pasteboard_restore_if_unchanged(view, expected_count) };
    crate::bridge::map_status(status)
}

fn free_view(view: BronzeNativeUtf8View) {
    let _ = unsafe { crate::abi::bronze_native_utf8_free(view) };
}

fn take_utf8(view: BronzeNativeUtf8View) -> Option<String> {
    let text = if view.ptr.is_null() {
        (view.len == 0).then(String::new)
    } else {
        let slice = unsafe { std::slice::from_raw_parts(view.ptr, view.len as usize) };
        String::from_utf8(slice.to_vec()).ok()
    };
    free_view(view);
    text
}

fn take_bytes(view: BronzeNativeUtf8View) -> Vec<u8> {
    let bytes = if view.ptr.is_null() {
        Vec::new()
    } else {
        let slice = unsafe { std::slice::from_raw_parts(view.ptr, view.len as usize) };
        slice.to_vec()
    };
    free_view(view);
    bytes
}

#[cfg(test)]
mod pasteboard_tests {
    use super::*;

    #[test]
    fn stub_pasteboard_is_degraded_and_omits_body() {
        let err = native_pasteboard_write("secret-plain", "<p>secret-html</p>").expect_err("stub");
        assert_eq!(err, NativeError::Degraded);
        assert!(!format!("{err:?}").contains("secret"));
        let swift = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../native/macos/BronzeNative/Sources/BronzeNative/PasteboardABI.swift"
        ));
        assert!(swift.contains("writeObjects"));
        assert!(swift.contains("forType: .string"));
        assert!(!swift.contains("wrotePlain && wroteHtml"));
        assert!(swift.contains("bronze_native_bounded_copy_read"));
        assert!(swift.contains("bronze_native_pasteboard_restore_if_unchanged"));
        assert!(swift.contains("0x42524E5A434F5059"));
        assert!(swift.contains("changeCount"));
        assert!(!swift.contains("URLSession"));
        assert!(!swift.contains("bronzeOnAppKitCopy"));
        let copy_fn = swift
            .split("public func bronze_native_bounded_copy_read")
            .nth(1)
            .expect("copy");
        let copy_end = copy_fn.find("@_silgen_name").unwrap_or(copy_fn.len());
        let copy_body = &copy_fn[..copy_end];
        assert!(copy_body.contains("Thread.isMainThread"));
        assert!(copy_body.contains("BRONZE_STATUS_DEGRADED"));
        assert!(!copy_body.contains("Thread.sleep"));
        assert!(!copy_body.contains("activate"));
        let bounded = swift
            .split("func bronzeBoundedCopyRead")
            .nth(1)
            .expect("bounded");
        let bounded_end = bounded.find("\nprivate func ").unwrap_or(bounded.len());
        let bounded_body = &bounded[..bounded_end];
        assert!(bounded_body.contains("Thread.isMainThread"));
        assert!(bounded_body.contains("BRONZE_STATUS_DEGRADED"));
        assert!(!bounded_body.contains("Thread.sleep"));
        assert!(!bounded_body.contains("activate"));
        let wait = swift
            .split("func waitStableGeneration")
            .nth(1)
            .expect("wait");
        assert!(wait.contains("Thread.sleep"));
        assert!(wait.contains("Thread.isMainThread"));
        let err = native_bounded_copy_read(1).expect_err("stub copy");
        assert_eq!(err, NativeError::Degraded);
        assert!(!format!("{err:?}").contains("secret"));
        native_pasteboard_restore_if_unchanged(&[], 3).expect("stub skip");
    }
}
