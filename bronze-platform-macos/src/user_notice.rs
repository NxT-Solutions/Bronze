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
            "/../native/macos/BronzeNative/Sources/BronzeNative/UserNoticeABI.swift"
        ));
        assert!(swift.contains("bronze_native_deliver_user_notice"));
        assert!(swift.contains("bronze_native_notification_authorization_status"));
        assert!(swift.contains("bronze_native_request_notification_authorization"));
        assert!(swift.contains("UNUserNotificationCenter"));
        assert!(swift.contains("requestAuthorization"));
        assert!(swift.contains(".denied"));
        assert!(swift.contains("willPresent"));
        assert!(swift.contains("announcementRequested"));
        assert!(!swift.contains("NSUserNotification"));
        assert!(!swift.contains("URLSession"));
        assert!(!swift.contains("http://"));
        assert!(!swift.contains("https://"));
        let tap = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        ));
        assert!(!tap.contains("bronze_native_deliver_user_notice"));
        assert!(!tap.contains("bronze_native_request_notification_authorization"));
        assert!(!tap.contains("UNUserNotificationCenter"));
        assert!(!tap.contains("UserNotifications"));
        assert!(!tap.contains("NSUserNotification"));
    }
}
