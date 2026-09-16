//! Rust-owned backup/export/import picker (story 7.4, DAT-002, DAT-003, QUE-008).

use bronze_storage::{BackupSchedule, SECRET_BODY_WARNING_KEY};
use std::path::Path;

pub const PICKER_OWNER_RUST: bool = true;
pub const WEBVIEW_PATHS_ALLOWED: bool = false;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathSource {
    RustPicker,
    WebView,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortabilityError {
    WebViewPathRejected,
    ScheduleForbidden,
    EmptyPath,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestorePreview {
    pub accepted: bool,
    pub warning_keys: Vec<&'static str>,
}

pub fn accept_native_path(source: PathSource, path: &str) -> Result<&Path, PortabilityError> {
    if matches!(source, PathSource::WebView) || !PICKER_OWNER_RUST {
        return Err(PortabilityError::WebViewPathRejected);
    }
    if path.is_empty() {
        return Err(PortabilityError::EmptyPath);
    }
    Ok(Path::new(path))
}

pub fn parse_backup_schedule(raw: &str) -> Result<BackupSchedule, PortabilityError> {
    match raw {
        "daily" => Ok(BackupSchedule::Daily),
        "weekly" => Ok(BackupSchedule::Weekly),
        "off" | "manual" | "manual-only" => Err(PortabilityError::ScheduleForbidden),
        _ => Err(PortabilityError::ScheduleForbidden),
    }
}

pub fn queue_export_warning_key() -> &'static str {
    SECRET_BODY_WARNING_KEY
}

pub fn preview_restore(source: PathSource, path: &str) -> Result<RestorePreview, PortabilityError> {
    let accepted_path = accept_native_path(source, path)?;
    Ok(RestorePreview {
        accepted: accepted_path.exists() || !accepted_path.as_os_str().is_empty(),
        warning_keys: vec![queue_export_warning_key()],
    })
}

#[cfg(test)]
mod portability_tests {
    use super::*;
    use crate::{window_allows, WindowCommand, WindowKind};
    use bronze_storage::{BackupSchedule, SECRET_BODY_WARNING_KEY};

    #[test]
    fn portability_rust_picker_rejects_webview_paths_and_off_schedule() {
        assert!(PICKER_OWNER_RUST);
        assert!(!WEBVIEW_PATHS_ALLOWED);
        assert_eq!(
            accept_native_path(PathSource::WebView, "/tmp/out").unwrap_err(),
            PortabilityError::WebViewPathRejected
        );
        assert!(accept_native_path(PathSource::RustPicker, "/tmp/out").is_ok());
        assert_eq!(
            parse_backup_schedule("off").unwrap_err(),
            PortabilityError::ScheduleForbidden
        );
        assert_eq!(
            parse_backup_schedule("daily").unwrap(),
            BackupSchedule::Daily
        );
        assert_eq!(queue_export_warning_key(), SECRET_BODY_WARNING_KEY);
        assert!(window_allows(WindowKind::Library, WindowCommand::Backup));
        assert!(window_allows(WindowKind::Library, WindowCommand::Export));
        assert!(window_allows(WindowKind::Library, WindowCommand::Import));
        assert!(!window_allows(WindowKind::Quick, WindowCommand::Backup));
        let preview = preview_restore(PathSource::RustPicker, "/tmp/restore").expect("preview");
        assert!(preview.warning_keys.contains(&SECRET_BODY_WARNING_KEY));
    }
}
