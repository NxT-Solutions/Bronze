use crate::abi::{
    BronzeNativeUtf8View, BRONZE_STATUS_CANCELLED, BRONZE_STATUS_NOT_FOUND, BRONZE_STATUS_OK,
};
use crate::bridge::NativeError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoticeAuthorization {
    Healthy,
    Denied,
    NotRequested,
    Unavailable,
}

impl NoticeAuthorization {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Denied => "denied",
            Self::NotRequested => "not_requested",
            Self::Unavailable => "unavailable",
        }
    }

    fn from_status(status: u32) -> Self {
        match status {
            BRONZE_STATUS_OK => Self::Healthy,
            BRONZE_STATUS_CANCELLED => Self::Denied,
            BRONZE_STATUS_NOT_FOUND => Self::NotRequested,
            _ => Self::Unavailable,
        }
    }
}

pub fn is_safe_notice_text(text: &str) -> bool {
    let trimmed = text.trim();
    !trimmed.is_empty()
        && trimmed.len() <= 200
        && !trimmed.contains('/')
        && !trimmed.contains('\\')
        && !trimmed.contains('\0')
        && !trimmed.contains("..")
}

pub fn is_safe_item_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

pub fn set_notice_item(id: &str) -> Result<(), NativeError> {
    if !is_safe_item_id(id) {
        return Err(NativeError::Degraded);
    }
    let view = BronzeNativeUtf8View {
        ptr: id.as_ptr(),
        len: id.len() as u64,
    };
    let status = unsafe { crate::abi::bronze_native_set_notice_item(view) };
    crate::bridge::map_status(status)
}

pub fn install_notice_click_hook(hook: unsafe extern "C" fn(*const u8, u64)) {
    unsafe {
        crate::abi::bronze_native_set_notice_click_hook(hook);
    }
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

pub fn notification_authorization_status() -> NoticeAuthorization {
    NoticeAuthorization::from_status(unsafe {
        crate::abi::bronze_native_notification_authorization_status()
    })
}

pub fn request_notification_authorization() -> NoticeAuthorization {
    NoticeAuthorization::from_status(unsafe {
        crate::abi::bronze_native_request_notification_authorization()
    })
}

pub fn prompt_notification_authorization_if_needed() -> NoticeAuthorization {
    match notification_authorization_status() {
        NoticeAuthorization::NotRequested => request_notification_authorization(),
        other => other,
    }
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
        assert_eq!(
            notification_authorization_status(),
            NoticeAuthorization::Unavailable
        );
        assert_eq!(
            prompt_notification_authorization_if_needed(),
            NoticeAuthorization::Unavailable
        );
        assert_eq!(NoticeAuthorization::Denied.as_str(), "denied");
        assert_eq!(NoticeAuthorization::NotRequested.as_str(), "not_requested");
        let swift = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../native/macos/BronzeNative/Sources/BronzeNative/UserNoticeABI.swift"
        ));
        assert!(swift.contains("bronze_native_deliver_user_notice"));
        assert!(swift.contains("bronze_native_notification_authorization_status"));
        assert!(swift.contains("bronze_native_request_notification_authorization"));
        assert!(swift.contains("UNUserNotificationCenter"));
        assert!(swift.contains("requestAuthorization"));
        assert!(swift.contains(".denied"));
        assert!(swift.contains("willPresent"));
        assert!(swift.contains("didReceive"));
        assert!(swift.contains("bronze_native_set_notice_item"));
        assert!(swift.contains("bronze_native_set_notice_click_hook"));
        assert!(swift.contains("bronzeItemId"));
        assert!(swift.contains("announcementRequested"));
        assert!(swift.contains("bundledNotificationCenter"));
        assert!(swift.contains("pathExtension == \"app\""));
        assert!(swift.contains("postLegacyNotice"));
        assert!(swift.contains("BronzeNotice.app"));
        assert!(swift.contains("noticeHelperApp"));
        assert!(swift.contains("/usr/bin/open"));
        assert!(swift.contains("\"-n\""));
        assert!(swift.contains("\"-g\""));
        assert!(swift.contains("\"--args\""));
        assert!(swift.contains("interruptionLevel"));
        assert!(swift.contains("UNTimeIntervalNotificationTrigger"));
        assert!(!swift.contains("display notification"));
        assert!(!swift.contains("osascript"));
        assert!(!swift.contains("NSUserNotificationCenter"));
        assert!(!swift.contains("NSUserNotification()"));
        assert!(!swift.contains("URLSession"));
        assert!(!swift.contains("http://"));
        assert!(!swift.contains("https://"));
        let helper = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../native/macos/BronzeNative/Sources/BronzeNotice/main.swift"
        ));
        assert!(helper.contains("UNUserNotificationCenter"));
        assert!(helper.contains("requestAuthorization"));
        assert!(helper.contains("willPresent"));
        assert!(helper.contains("didReceive"));
        assert!(helper.contains("bronzeItemId"));
        assert!(helper.contains("app.bronze.desktop.notice-activate"));
        assert!(helper.contains("isSafeNoticeText"));
        assert!(helper.contains("noticePayload"));
        assert!(helper.contains("-psn_"));
        assert!(helper.contains("applicationWillFinishLaunching"));
        assert!(helper.contains("NSApp.activate"));
        assert!(helper.contains("setActivationPolicy"));
        assert!(helper.contains("interruptionLevel"));
        assert!(helper.contains("UNTimeIntervalNotificationTrigger"));
        assert!(helper.contains("Notifications-Settings.extension"));
        assert!(!helper.contains("/tmp/"));
        assert!(!helper.contains("osascript"));
        assert!(!helper.contains("display notification"));
        assert!(!helper.contains("URLSession"));
        assert!(!helper.contains("http://"));
        assert!(!helper.contains("https://"));
        assert!(!helper.contains("UNNotificationAttachment"));
        assert!(!helper.contains("content.attachments"));
        assert!(!helper.contains("NSImage"));
        assert!(!helper.contains("32x32"));
        assert!(!helper.contains("128x128"));
        assert!(!swift.contains("UNNotificationAttachment"));
        assert!(!swift.contains("content.attachments"));
        assert!(!swift.contains("NSImage"));
        assert!(!swift.contains("32x32"));
        assert!(!swift.contains("128x128"));
        let build = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../apps/desktop/src-tauri/build.rs"
        ));
        let wrap_at = build
            .find("fn wrap_notice_helper")
            .expect("notice helper wrapper");
        let wrap = &build[wrap_at..];
        let wrap_end = wrap.find("\nfn ").unwrap_or(wrap.len());
        let wrap = &wrap[..wrap_end];
        assert!(wrap.contains("icons/icon.icns") || build.contains("icons/icon.icns"));
        assert!(wrap.contains("AppIcon.icns"));
        assert!(wrap.contains("notification banner must use the Dock icon.icns"));
        assert!(wrap.contains("AppIcon.icns must match icon.icns"));
        assert!(wrap.contains("dock-icon-fill.py"));
        assert!(!wrap.contains("32x32"));
        assert!(!wrap.contains("128x128"));
        let call_at = build
            .find("wrap_notice_helper(")
            .expect("notice helper call");
        let call = &build[call_at..call_at + 180];
        assert!(
            call.contains("icons/icon.icns"),
            "BronzeNotice must copy the Dock icon.icns, got {call}"
        );
        let tap = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        ));
        assert!(!tap.contains("bronze_native_deliver_user_notice"));
        assert!(!tap.contains("bronze_native_request_notification_authorization"));
        assert!(!tap.contains("bronze_native_apply_login_item"));
        assert!(!tap.contains("SMAppService"));
        assert!(!tap.contains("UNUserNotificationCenter"));
        assert!(!tap.contains("UserNotifications"));
        assert!(!tap.contains("NSUserNotification"));
        assert!(!tap.contains("osascript"));
        assert!(!tap.contains("BronzeNotice"));
        assert!(!tap.contains("display notification"));
        assert!(!tap.contains("/usr/bin/open"));
    }
}
