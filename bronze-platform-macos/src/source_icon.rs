use crate::abi::{BronzeNativeUtf8View, BRONZE_STATUS_OK};

pub fn native_bundle_id_for_pid(pid: i32) -> Option<String> {
    if pid <= 0 {
        return None;
    }
    let mut out = BronzeNativeUtf8View {
        ptr: std::ptr::null(),
        len: 0,
    };
    let status = unsafe { crate::abi::bronze_native_bundle_id_for_pid(pid, &mut out) };
    take_utf8(status, out)
}

// PNG octets reuse the ABI view; they are not UTF-8. Free with bronze_native_utf8_free.
pub fn native_app_icon_png(bundle_or_name: &str) -> Option<Vec<u8>> {
    if bundle_or_name.is_empty() {
        return None;
    }
    let view = BronzeNativeUtf8View {
        ptr: bundle_or_name.as_ptr(),
        len: bundle_or_name.len() as u64,
    };
    let mut out = BronzeNativeUtf8View {
        ptr: std::ptr::null(),
        len: 0,
    };
    let status = unsafe { crate::abi::bronze_native_app_icon_png(view, &mut out) };
    if status != BRONZE_STATUS_OK {
        return None;
    }
    let bytes = if out.ptr.is_null() {
        (out.len == 0).then(Vec::new)
    } else {
        let slice = unsafe { std::slice::from_raw_parts(out.ptr, out.len as usize) };
        Some(slice.to_vec())
    };
    let _ = unsafe { crate::abi::bronze_native_utf8_free(out) };
    bytes.filter(|png| !png.is_empty())
}

fn take_utf8(status: u32, out: BronzeNativeUtf8View) -> Option<String> {
    if status != BRONZE_STATUS_OK {
        return None;
    }
    let copied = if out.ptr.is_null() {
        (out.len == 0).then(String::new)
    } else {
        let slice = unsafe { std::slice::from_raw_parts(out.ptr, out.len as usize) };
        String::from_utf8(slice.to_vec()).ok()
    };
    let _ = unsafe { crate::abi::bronze_native_utf8_free(out) };
    copied.filter(|text| !text.is_empty())
}

#[cfg(test)]
mod source_icon_tests {
    use super::*;

    #[test]
    fn stub_bundle_and_icon_are_unavailable_without_payload() {
        assert_eq!(native_bundle_id_for_pid(1), None);
        assert_eq!(native_app_icon_png("com.apple.TextEdit"), None);
        assert!(!format!("{:?}", native_app_icon_png("secret-bundle")).contains("secret"));
        assert!(!format!("{:?}", native_bundle_id_for_pid(42)).contains("42"));
        let swift = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/SourceIconABI.swift"
        ));
        assert!(!swift.contains("URLSession"));
        assert!(!swift.contains("http://"));
        assert!(!swift.contains("https://"));
        assert!(swift.contains("caseInsensitiveCompare"));
        assert!(swift.contains("/Applications"));
        assert!(swift.contains("sourceOver"));
        assert!(!swift.contains("com.mitchellh.ghostty"));
        let tap = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        ));
        assert!(!tap.contains("bronze_native_app_icon_png"));
        assert!(!tap.contains("bronze_native_bundle_id_for_pid"));
        assert!(!tap.contains("bronze_native_list_installed_apps"));
        assert!(!tap.contains("bronze_native_pick_installed_app"));
        assert!(!tap.contains("bronze_native_deliver_user_notice"));
        assert!(!tap.contains("bronze_native_request_notification_authorization"));
        assert!(!tap.contains("UNUserNotificationCenter"));
        assert!(!tap.contains("NSOpenPanel"));
        assert!(!tap.contains("bronze_native_pasteboard_write"));
    }
}
