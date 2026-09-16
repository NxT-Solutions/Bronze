//! bronze-settings (SET-001, WIN-005, docs/12)

mod export;
mod permission;
mod schema;
mod shortcuts;

pub use export::{
    export_flags_sensitive_key, export_settings, settings_key_exportable,
    value_looks_like_machine_path, SettingsExportPreview,
};
pub use permission::{PermissionSnapshot, PermissionState};
pub use schema::{
    search_settings, BackupSchedule, SchemaError, SettingsField, SettingsGroup, SettingsV1,
    ShortcutActionId, ShortcutBinding, SCHEMA_VERSION, SETTINGS_FIELDS,
};
pub use shortcuts::{
    capture_alternatives_ok, recorder_swallows, skip_test_marks_untested, CaptureAlternatives,
    FakeRegistrar, NativeRegistrar, RecorderEvent, RecorderIgnore, RegisterError, ShortcutRegistry,
};
