//! bronze-settings (SET-001, WIN-005, docs/12)

mod export;
mod health;
mod permission;
mod schema;
mod shortcuts;

pub use export::{
    export_flags_sensitive_key, export_settings, export_settings_document, parse_settings_import,
    settings_json_looks_like_json, settings_key_exportable, value_looks_like_machine_path,
    SettingsExportDocument, SettingsExportPreview, SettingsImportError, SettingsProfileExport,
    SETTINGS_EXPORT_FORMAT, SETTINGS_EXPORT_MAX_BYTES, SETTINGS_EXPORT_VERSION,
};
pub use health::{
    capabilities_independent, health_rows, manual_composer_available, privacy_settings_url,
    run_content_free_self_test, screen_recording_row, HealthError, PermissionCapability,
    PermissionHealthRow, PermissionUsage, LAUNCH_LOOP_PROMPTING, SCREEN_RECORDING_USED,
    SELF_TEST_CAPTURES_CONTENT,
};
pub use permission::{PermissionSnapshot, PermissionState};
pub use schema::{
    persisted_locale_allowed, search_settings, BackupSchedule, ClipboardFallback, Modifier,
    ModifierSide, MotionPref, PanelEdge, SchemaError, SettingsField, SettingsGroup, SettingsV1,
    ShortcutActionId, ShortcutBinding, TestedState, TitleModelId, TriggerKind, DEFAULT_GAP_MS,
    DEFAULT_MAX_HOLD_MS, PERSISTED_LOCALE_TAGS, SCHEMA_VERSION, SETTINGS_FIELDS,
};
pub use shortcuts::{
    apply_recorded_double_tap_timing, capture_alternatives_ok, default_shortcut_binding,
    effective_tap_count, is_default_binding, live_shift_tap_count, logical_key_allowed,
    recorded_double_tap_allowed, recorder_swallows, shortcut_scope, skip_test_marks_untested,
    CaptureAlternatives, FakeRegistrar, NativeRegistrar, RecorderEvent, RecorderIgnore,
    RegisterError, ShortcutRegistry, ShortcutScope, MAX_MODIFIER_TAPS,
};
