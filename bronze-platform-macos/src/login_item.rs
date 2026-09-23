use crate::abi::{
    BRONZE_STATUS_CANCELLED, BRONZE_STATUS_DEGRADED, BRONZE_STATUS_NOT_FOUND, BRONZE_STATUS_OK,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoginItemStatus {
    Enabled,
    NotRegistered,
    RequiresApproval,
    Unavailable,
}

impl LoginItemStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::NotRegistered => "not_registered",
            Self::RequiresApproval => "requires_approval",
            Self::Unavailable => "unavailable",
        }
    }

    fn from_status(status: u32) -> Self {
        match status {
            BRONZE_STATUS_OK => Self::Enabled,
            BRONZE_STATUS_NOT_FOUND => Self::NotRegistered,
            BRONZE_STATUS_CANCELLED => Self::RequiresApproval,
            BRONZE_STATUS_DEGRADED => Self::Unavailable,
            _ => Self::Unavailable,
        }
    }
}

pub fn login_item_status() -> LoginItemStatus {
    LoginItemStatus::from_status(unsafe { crate::abi::bronze_native_login_item_status() })
}

pub fn apply_login_item(enabled: bool) -> LoginItemStatus {
    LoginItemStatus::from_status(unsafe {
        crate::abi::bronze_native_apply_login_item(u32::from(enabled))
    })
}

#[cfg(test)]
mod login_item_tests {
    use super::*;

    #[test]
    fn stub_maps_status_without_live_smappservice() {
        assert_eq!(login_item_status(), LoginItemStatus::Unavailable);
        assert_eq!(apply_login_item(true), LoginItemStatus::Unavailable);
        assert_eq!(apply_login_item(false), LoginItemStatus::Unavailable);
        assert_eq!(
            LoginItemStatus::from_status(BRONZE_STATUS_OK),
            LoginItemStatus::Enabled
        );
        assert_eq!(
            LoginItemStatus::from_status(BRONZE_STATUS_NOT_FOUND),
            LoginItemStatus::NotRegistered
        );
        assert_eq!(
            LoginItemStatus::from_status(BRONZE_STATUS_CANCELLED),
            LoginItemStatus::RequiresApproval
        );
        assert_eq!(
            LoginItemStatus::from_status(BRONZE_STATUS_DEGRADED),
            LoginItemStatus::Unavailable
        );
        assert_eq!(
            LoginItemStatus::from_status(99),
            LoginItemStatus::Unavailable
        );
        assert_eq!(LoginItemStatus::Enabled.as_str(), "enabled");
        assert_eq!(LoginItemStatus::NotRegistered.as_str(), "not_registered");
        assert_eq!(
            LoginItemStatus::RequiresApproval.as_str(),
            "requires_approval"
        );
        assert_eq!(LoginItemStatus::Unavailable.as_str(), "unavailable");
        let swift = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/LoginItemABI.swift"
        ));
        assert!(swift.contains("bronze_native_login_item_status"));
        assert!(swift.contains("bronze_native_apply_login_item"));
        assert!(swift.contains("SMAppService"));
        assert!(swift.contains("ServiceManagement"));
        assert!(swift.contains("mainApp"));
        assert!(swift.contains("try service.register()"));
        assert!(swift.contains("try service.unregister()"));
        assert!(swift.contains(".requiresApproval"));
        assert!(swift.contains(".notRegistered"));
        assert!(swift.contains(".notFound"));
        assert!(swift.contains(".enabled"));
        assert!(swift.contains("pathExtension == \"app\""));
        assert!(swift.contains("bundledMainApp"));
        assert!(swift.contains("LaunchAgents"));
        assert!(swift.contains("app.bronze.desktop.login"));
        assert!(swift.contains("removeLeftoverDebugLoginAgent"));
        assert!(swift.contains("removeLoginAgentIfThisProcessOwnsLogin"));
        assert!(swift.contains("bootoutLoginAgent"));
        assert!(swift.contains("/bin/launchctl"));
        assert!(swift.contains("return BRONZE_STATUS_DEGRADED"));
        assert!(!swift.contains("writeLoginAgent"));
        assert!(!swift.contains("bootstrapLoginAgent"));
        assert!(!swift.contains("launchAgentIsLoaded"));
        assert!(!swift.contains("RunAtLoad"));
        assert!(!swift.contains("URLSession"));
        assert!(!swift.contains("http://"));
        assert!(!swift.contains("https://"));
        assert!(!swift.contains("LSSharedFileList"));
        assert!(!swift.contains("SMLoginItemSetEnabled"));
        let package = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Package.swift"
        ));
        assert!(package.contains("ServiceManagement"));
        let build = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../apps/desktop/src-tauri/build.rs"
        ));
        assert!(build.contains("framework=ServiceManagement"));
        let header = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/include/BronzeNative.h"
        ));
        assert!(header.contains("bronze_native_login_item_status"));
        assert!(header.contains("bronze_native_apply_login_item"));
        let tap = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        ));
        assert!(!tap.contains("SMAppService"));
        assert!(!tap.contains("ServiceManagement"));
        assert!(!tap.contains("bronze_native_apply_login_item"));
        assert!(!tap.contains("bronze_native_login_item_status"));
        assert!(!tap.contains("launchAtLogin"));
        assert!(!tap.contains("apply_login_item"));
        assert!(!tap.contains("LaunchAgents"));
        assert!(!tap.contains("launchctl"));
        assert!(!tap.contains("LSSharedFileList"));
        assert!(!tap.contains("SMLoginItemSetEnabled"));
    }
}
