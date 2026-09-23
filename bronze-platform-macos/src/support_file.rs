use crate::abi::{BronzeNativeUtf8View, BRONZE_STATUS_CANCELLED, BRONZE_STATUS_OK};
use crate::settings_file::{PickSettingsFile, SettingsFileError};
use std::path::{Component, Path, PathBuf};

pub fn try_pick_support_export_path() -> PickSettingsFile {
    native_pick(crate::abi::bronze_native_pick_support_export_path)
}

pub fn accept_support_file_path(raw: &str) -> Result<PathBuf, SettingsFileError> {
    if raw.is_empty() || raw.contains('\0') {
        return Err(SettingsFileError::Invalid);
    }
    let path = Path::new(raw);
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(SettingsFileError::Invalid);
    }
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    if !ext.eq_ignore_ascii_case("txt") {
        return Err(SettingsFileError::Invalid);
    }
    Ok(path.to_path_buf())
}

fn native_pick(
    status_fn: unsafe extern "C" fn(*mut BronzeNativeUtf8View) -> u32,
) -> PickSettingsFile {
    let mut out = BronzeNativeUtf8View {
        ptr: std::ptr::null(),
        len: 0,
    };
    let status = unsafe { status_fn(&mut out) };
    if status == BRONZE_STATUS_CANCELLED {
        return PickSettingsFile::Cancelled;
    }
    if status != BRONZE_STATUS_OK {
        if !out.ptr.is_null() {
            let _ = unsafe { crate::abi::bronze_native_utf8_free(out) };
        }
        return PickSettingsFile::Unavailable;
    }
    let copied = if out.ptr.is_null() {
        None
    } else {
        let slice = unsafe { std::slice::from_raw_parts(out.ptr, out.len as usize) };
        let text = String::from_utf8(slice.to_vec()).ok();
        let _ = unsafe { crate::abi::bronze_native_utf8_free(out) };
        text
    };
    match copied.and_then(|raw| accept_support_file_path(&raw).ok()) {
        Some(path) => PickSettingsFile::Picked(path),
        None => PickSettingsFile::Unavailable,
    }
}

#[cfg(test)]
mod support_file_tests {
    use super::*;

    #[test]
    fn support_file_path_and_picker_stay_rust_owned() {
        assert_eq!(
            accept_support_file_path("../etc/passwd.txt").unwrap_err(),
            SettingsFileError::Invalid
        );
        assert_eq!(
            accept_support_file_path("/tmp/support.json").unwrap_err(),
            SettingsFileError::Invalid
        );
        assert!(accept_support_file_path("/tmp/bronze-support.txt").is_ok());
        assert_eq!(
            try_pick_support_export_path(),
            PickSettingsFile::Unavailable
        );
        let swift = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/SourceIconABI.swift"
        ));
        assert!(swift.contains("bronze_native_pick_support_export_path"));
        assert!(swift.contains("bronze-support.txt"));
        assert!(swift.contains("NSSavePanel"));
        let tap = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        ));
        assert!(!tap.contains("bronze_native_pick_support_export_path"));
        assert!(!tap.contains("NSSavePanel"));
    }
}
