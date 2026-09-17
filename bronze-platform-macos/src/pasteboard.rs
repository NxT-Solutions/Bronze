use crate::abi::BronzeNativeUtf8View;
use crate::bridge::NativeError;

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
            "/../native/macos/BronzeNative/Sources/BronzeNative/PasteboardABI.swift"
        ));
        assert!(swift.contains("writeObjects"));
        assert!(swift.contains("forType: .string"));
        assert!(!swift.contains("wrotePlain && wroteHtml"));
    }
}
