//! Settings export preview (story 7.1, SET-001). Excludes credentials/tokens/paths.

use crate::schema::SettingsV1;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

const FORBIDDEN_KEY_FRAGMENTS: [&str; 6] = [
    "credential",
    "token",
    "path",
    "secret",
    "installidentity",
    "diagnosticevent",
];

const SENSITIVE_LITERAL_KEYS: [&str; 3] = [
    "privacy.excludedBundleIds",
    "privacy.appPolicies",
    "capture.standardChord",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettingsExportPreview {
    pub payload: Value,
    pub sensitive_literal_keys: Vec<String>,
    pub excluded_keys: Vec<String>,
}

pub fn settings_key_exportable(key: &str) -> bool {
    let lower = key.to_ascii_lowercase().replace(['.', '_', '-'], "");
    !FORBIDDEN_KEY_FRAGMENTS
        .iter()
        .any(|frag| lower.contains(frag))
}

pub fn value_looks_like_machine_path(value: &str) -> bool {
    let trimmed = value.trim().trim_matches('"');
    trimmed.starts_with('/') || trimmed.contains(":\\")
}

pub fn export_settings(
    settings: &SettingsV1,
    extra_raw: &BTreeMap<String, String>,
) -> Result<SettingsExportPreview, crate::schema::SchemaError> {
    settings.validate()?;
    let typed =
        serde_json::to_value(settings).map_err(|_| crate::schema::SchemaError::UnknownVersion)?;
    let mut payload = match typed {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    let mut excluded_keys = Vec::new();
    for (key, value) in extra_raw {
        if !settings_key_exportable(key) || value_looks_like_machine_path(value) {
            excluded_keys.push(key.clone());
            continue;
        }
        let parsed = serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.clone()));
        payload.insert(key.clone(), parsed);
    }
    excluded_keys.sort();
    let mut sensitive_literal_keys = Vec::new();
    if !settings.privacy.excluded_bundle_ids.is_empty() {
        sensitive_literal_keys.push("privacy.excludedBundleIds".into());
    }
    if !settings.privacy.app_policies.is_empty() {
        sensitive_literal_keys.push("privacy.appPolicies".into());
    }
    if settings.capture.standard_chord.enabled
        || settings.capture.standard_chord.trigger != crate::schema::TriggerKind::Disabled
    {
        sensitive_literal_keys.push("capture.standardChord".into());
    }
    Ok(SettingsExportPreview {
        payload: Value::Object(payload),
        sensitive_literal_keys,
        excluded_keys,
    })
}

pub fn export_flags_sensitive_key(key: &str) -> bool {
    SENSITIVE_LITERAL_KEYS.contains(&key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{
        AppPolicy, InheritAllowDeny, InheritAllowDenyAsk, InheritProvenance, SettingsV1,
        TriggerKind,
    };

    #[test]
    fn settings_export_excludes_credentials_tokens_paths() {
        let settings = SettingsV1::defaults();
        let mut extra = BTreeMap::new();
        extra.insert("theme".into(), "\"dark\"".into());
        extra.insert("permissionToken".into(), "\"abc\"".into());
        extra.insert("installIdentity".into(), "\"machine-1\"".into());
        extra.insert("logPath".into(), "\"/Users/me/bronze.log\"".into());
        extra.insert("appCredential".into(), "\"secret\"".into());
        extra.insert("diagnosticEvent".into(), "\"tap\"".into());
        extra.insert("okPref".into(), "\"safe\"".into());
        extra.insert("homePath".into(), "\"/tmp/x\"".into());
        let preview = export_settings(&settings, &extra).expect("export");
        let blob = preview.payload.to_string();
        assert!(blob.contains("backupSchedule"));
        assert!(blob.contains("daily"));
        assert!(blob.contains("\"theme\""));
        assert!(blob.contains("okPref"));
        assert!(!blob.contains("permissionToken"));
        assert!(!blob.contains("installIdentity"));
        assert!(!blob.contains("abc"));
        assert!(!blob.contains("/Users/me"));
        assert!(!blob.contains("appCredential"));
        assert!(!blob.contains("diagnosticEvent"));
        assert!(!blob.contains("/tmp/x"));
        assert!(preview.excluded_keys.iter().any(|k| k == "permissionToken"));
        assert!(preview.excluded_keys.iter().any(|k| k == "logPath"));
    }

    #[test]
    fn settings_export_preview_flags_sensitive_literals() {
        let mut settings = SettingsV1::defaults();
        settings
            .privacy
            .excluded_bundle_ids
            .push("com.bank.app".into());
        settings.privacy.app_policies.insert(
            "com.example.vault".into(),
            AppPolicy {
                capture: InheritAllowDeny::Deny,
                synthetic_fallback: InheritAllowDenyAsk::Deny,
                provenance: InheritProvenance::None,
            },
        );
        settings.capture.standard_chord.enabled = true;
        settings.capture.standard_chord.trigger = TriggerKind::Accelerator;
        let preview = export_settings(&settings, &BTreeMap::new()).expect("export");
        assert!(preview
            .sensitive_literal_keys
            .contains(&"privacy.excludedBundleIds".into()));
        assert!(preview
            .sensitive_literal_keys
            .contains(&"privacy.appPolicies".into()));
        assert!(preview
            .sensitive_literal_keys
            .contains(&"capture.standardChord".into()));
        assert!(export_flags_sensitive_key("privacy.appPolicies"));
        let blob = preview.payload.to_string();
        assert!(blob.contains("com.bank.app"));
        assert!(!blob.contains("permissionToken"));
    }
}
