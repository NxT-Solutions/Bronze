use crate::abi::{BronzeNativeUtf8View, BRONZE_STATUS_CANCELLED, BRONZE_STATUS_OK};
use bronze_settings::{settings_json_looks_like_json, SETTINGS_EXPORT_MAX_BYTES};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsFileError {
    Invalid,
    TooLarge,
    NotJson,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PickSettingsFile {
    Picked(PathBuf),
    Cancelled,
    Unavailable,
}

pub fn try_pick_settings_export_path() -> PickSettingsFile {
    native_pick(crate::abi::bronze_native_pick_settings_export_path)
}

pub fn try_pick_settings_import_path() -> PickSettingsFile {
    native_pick(crate::abi::bronze_native_pick_settings_import_path)
}

pub fn accept_settings_file_path(raw: &str) -> Result<PathBuf, SettingsFileError> {
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
    if !ext.eq_ignore_ascii_case("json") {
        return Err(SettingsFileError::Invalid);
    }
    Ok(path.to_path_buf())
}

pub fn read_settings_import_bytes(path: &Path) -> Result<Vec<u8>, SettingsFileError> {
    let accepted = accept_settings_file_path(path.to_str().unwrap_or_default())?;
    let canonical = accepted
        .canonicalize()
        .map_err(|_| SettingsFileError::Invalid)?;
    let ext = canonical
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    if !ext.eq_ignore_ascii_case("json") {
        return Err(SettingsFileError::Invalid);
    }
    let meta = fs::metadata(&canonical).map_err(|_| SettingsFileError::Invalid)?;
    if !meta.is_file() {
        return Err(SettingsFileError::Invalid);
    }
    if meta.len() > SETTINGS_EXPORT_MAX_BYTES {
        return Err(SettingsFileError::TooLarge);
    }
    let bytes = fs::read(&canonical).map_err(|_| SettingsFileError::Invalid)?;
    if !settings_json_looks_like_json(&bytes) {
        return Err(SettingsFileError::NotJson);
    }
    Ok(bytes)
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
    match copied.and_then(|raw| accept_settings_file_path(&raw).ok()) {
        Some(path) => PickSettingsFile::Picked(path),
        None => PickSettingsFile::Unavailable,
    }
}

#[cfg(test)]
mod settings_file_tests {
    use super::*;

    #[test]
    fn settings_file_path_and_picker_stay_rust_owned() {
        assert_eq!(
            accept_settings_file_path("../etc/passwd.json").unwrap_err(),
            SettingsFileError::Invalid
        );
        assert_eq!(
            accept_settings_file_path("/tmp/settings.txt").unwrap_err(),
            SettingsFileError::Invalid
        );
        assert!(accept_settings_file_path("/tmp/bronze-settings.json").is_ok());
        assert_eq!(
            try_pick_settings_export_path(),
            PickSettingsFile::Unavailable
        );
        assert_eq!(
            try_pick_settings_import_path(),
            PickSettingsFile::Unavailable
        );
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bronze-settings-file-{nanos}"));
        fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("bronze-settings.json");
        fs::write(&path, r#"{"format":"bronze-settings","version":1}"#).expect("write");
        let bytes = read_settings_import_bytes(&path).expect("read");
        assert!(settings_json_looks_like_json(&bytes));
        fs::write(&path, "not-json").expect("overwrite");
        assert_eq!(
            read_settings_import_bytes(&path).unwrap_err(),
            SettingsFileError::NotJson
        );
        let swift = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/SourceIconABI.swift"
        ));
        assert!(swift.contains("bronze_native_pick_settings_export_path"));
        assert!(swift.contains("bronze_native_pick_settings_import_path"));
        assert!(swift.contains("NSSavePanel"));
        assert!(swift.contains("bronze-settings.json"));
        assert!(swift.contains("allowedFileTypes"));
        assert!(!swift.contains("URLSession"));
        let tap = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        ));
        assert!(!tap.contains("bronze_native_pick_settings_export_path"));
        assert!(!tap.contains("bronze_native_pick_settings_import_path"));
        assert!(!tap.contains("NSSavePanel"));
        assert!(!tap.contains("NSOpenPanel"));
        fs::remove_dir_all(&dir).expect("cleanup");
    }
}
