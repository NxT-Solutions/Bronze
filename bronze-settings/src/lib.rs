//! bronze-settings (SET-001, WIN-005, docs/12)

mod export;
mod health;
mod permission;
mod schema;
mod shortcuts;

pub use export::{
    export_flags_sensitive_key, export_settings, settings_key_exportable,
    value_looks_like_machine_path, SettingsExportPreview,
};
pub use health::{
    capabilities_independent, health_rows, manual_composer_available, privacy_settings_url,
    run_content_free_self_test, screen_recording_row, HealthError, PermissionCapability,
    PermissionHealthRow, PermissionUsage, LAUNCH_LOOP_PROMPTING, SCREEN_RECORDING_USED,
    SELF_TEST_CAPTURES_CONTENT,
};
pub use permission::{PermissionSnapshot, PermissionState};
pub use schema::{
    persisted_locale_allowed, search_settings, BackupSchedule, Modifier, SchemaError,
    SettingsField, SettingsGroup, SettingsV1, ShortcutActionId, ShortcutBinding, TestedState,
    TriggerKind, PERSISTED_LOCALE_TAGS, SCHEMA_VERSION, SETTINGS_FIELDS,
};
pub use shortcuts::{
    capture_alternatives_ok, default_shortcut_binding, is_default_binding, logical_key_allowed,
    recorder_swallows, shortcut_scope, skip_test_marks_untested, CaptureAlternatives,
    FakeRegistrar, NativeRegistrar, RecorderEvent, RecorderIgnore, RegisterError, ShortcutRegistry,
    ShortcutScope,
};
