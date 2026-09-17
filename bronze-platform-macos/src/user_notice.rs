use crate::abi::BronzeNativeUtf8View;
use crate::bridge::NativeError;

pub fn is_safe_notice_text(text: &str) -> bool {
    let trimmed = text.trim();
    !trimmed.is_empty()
        && trimmed.len() <= 200
        && !trimmed.contains('/')
        && !trimmed.contains('\\')
        && !trimmed.contains('\0')
        && !trimmed.contains("..")
}

pub fn try_deliver_user_notice(title: &str, body: &str) -> Result<(), NativeError> {
    if !is_safe_notice_text(title) || !is_safe_notice_text(body) {
        return Err(NativeError::Degraded);
    }
    let title_view = BronzeNativeUtf8View {
        ptr: title.as_ptr(),
        len: title.len() as u64,
    };
    let body_view = BronzeNativeUtf8View {
        ptr: body.as_ptr(),
        len: body.len() as u64,
    };
    let status = unsafe { crate::abi::bronze_native_deliver_user_notice(title_view, body_view) };
    crate::bridge::map_status(status)
}

#[cfg(test)]
mod user_notice_tests {
    use super::*;

    #[test]
    fn stub_notice_rejects_paths_and_stays_off_event_tap() {
        assert!(is_safe_notice_text("Captured to Bronze."));
        assert!(is_safe_notice_text("This app is excluded from capture."));
        assert!(!is_safe_notice_text(""));
        assert!(!is_safe_notice_text("/tmp/secret.txt"));
        assert!(!is_safe_notice_text(".. hidden"));
        assert_eq!(
            try_deliver_user_notice("Bronze", "Captured to Bronze.").unwrap_err(),
            NativeError::Degraded
        );
        assert_eq!(
            try_deliver_user_notice("/Applications/X.app", "ok").unwrap_err(),
            NativeError::Degraded
        );
        let err = try_deliver_user_notice("Bronze", "secret-body-must-not-leak");
        assert!(!format!("{err:?}").contains("secret-body"));
        let swift = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/UserNoticeABI.swift"
        ));
        assert!(swift.contains("bronze_native_deliver_user_notice"));
        assert!(swift.contains("NSUserNotification"));
        assert!(swift.contains("NSUserNotificationCenter"));
        assert!(swift.contains("announcementRequested"));
        assert!(!swift.contains("URLSession"));
        assert!(!swift.contains("http://"));
        assert!(!swift.contains("https://"));
        let tap = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        ));
        assert!(!tap.contains("bronze_native_deliver_user_notice"));
        assert!(!tap.contains("UNUserNotificationCenter"));
        assert!(!tap.contains("UserNotifications"));
    }
}
