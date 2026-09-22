//! Settings JSON export/import (SET-001). Excludes credentials, tokens, diagnostics, and paths.

use crate::schema::{SchemaError, SettingsV1, SETTINGS_FIELDS};
use crate::shortcuts::is_default_binding;
use crate::ShortcutBinding;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub const SETTINGS_EXPORT_FORMAT: &str = "bronze-settings";
pub const SETTINGS_EXPORT_VERSION: u32 = 1;
pub const SETTINGS_EXPORT_MAX_BYTES: u64 = 1_048_576;

const FORBIDDEN_KEY_FRAGMENTS: [&str; 6] = [
    "credential",
    "token",
    "path",
    "secret",
    "installidentity",
    "diagnosticevent",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsImportError {
    UnknownVersion,
    InvalidSchema,
    WrongFormat,
    TooLarge,
    InvalidJson,
    ForbiddenContent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsProfileExport {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub builtin_key: Option<String>,
    pub name: String,
    pub format: String,
    pub format_options: Value,
    pub source_policy: String,
    pub post_copy_action: String,
    pub advance_policy: String,
    pub revision: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsExportDocument {
    pub format: String,
    pub version: u32,
    pub exported_at: String,
    pub settings: SettingsV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcuts: Option<Vec<ShortcutBinding>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profiles: Option<Vec<SettingsProfileExport>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettingsExportPreview {
    pub payload: Value,
    pub included_categories: Vec<String>,
    pub included_fields: Vec<String>,
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

pub fn settings_json_looks_like_json(raw: &[u8]) -> bool {
    let trimmed = raw
        .iter()
        .copied()
        .skip_while(|byte| byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    trimmed.first().copied() == Some(b'{')
}

pub fn export_settings(
    settings: &SettingsV1,
    extra_raw: &BTreeMap<String, String>,
) -> Result<SettingsExportPreview, SchemaError> {
    export_settings_document(settings, &[], &[], extra_raw, "1970-01-01T00:00:00Z")
}

pub fn export_settings_document(
    settings: &SettingsV1,
    shortcuts: &[ShortcutBinding],
    profiles: &[SettingsProfileExport],
    extra_raw: &BTreeMap<String, String>,
    exported_at: &str,
) -> Result<SettingsExportPreview, SchemaError> {
    settings.validate()?;
    let typed = serde_json::to_value(settings).map_err(|_| SchemaError::UnknownVersion)?;
    let mut settings_map = match typed {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    let mut excluded_keys = Vec::new();
    sanitize_object(&mut settings_map, "", &mut excluded_keys);
    for (key, value) in extra_raw {
        if !settings_key_exportable(key) || value_looks_like_machine_path(value) {
            excluded_keys.push(key.clone());
            continue;
        }
        let parsed = serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.clone()));
        settings_map.insert(key.clone(), parsed);
    }
    let exported_shortcuts: Vec<ShortcutBinding> = shortcuts
        .iter()
        .cloned()
        .map(strip_ephemeral_shortcut)
        .collect();
    let exported_profiles: Vec<SettingsProfileExport> = profiles
        .iter()
        .filter(|profile| settings_profile_exportable(profile, &mut excluded_keys))
        .cloned()
        .collect();
    let document = SettingsExportDocument {
        format: SETTINGS_EXPORT_FORMAT.into(),
        version: SETTINGS_EXPORT_VERSION,
        exported_at: exported_at.into(),
        settings: settings.clone(),
        shortcuts: Some(exported_shortcuts.clone()),
        profiles: Some(exported_profiles.clone()),
    };
    let mut payload = serde_json::to_value(&document).map_err(|_| SchemaError::UnknownVersion)?;
    if let Value::Object(map) = &mut payload {
        map.insert("settings".into(), Value::Object(settings_map));
        if let Some(Value::Array(items)) = map.get_mut("shortcuts") {
            for item in items.iter_mut() {
                if let Value::Object(binding) = item {
                    binding.remove("tested");
                }
            }
        }
    }
    excluded_keys.sort();
    excluded_keys.dedup();
    let sensitive_literal_keys =
        collect_sensitive_keys(settings, &exported_shortcuts, &exported_profiles);
    let included_categories = vec![
        "general".into(),
        "capture".into(),
        "panel".into(),
        "copy".into(),
        "privacy".into(),
        "data".into(),
        "accessibility".into(),
        "shortcuts".into(),
        "profiles".into(),
    ];
    let mut included_fields: Vec<String> = SETTINGS_FIELDS
        .iter()
        .map(|field| field.id.into())
        .collect();
    included_fields.push("shortcuts".into());
    included_fields.push("profiles".into());
    Ok(SettingsExportPreview {
        payload,
        included_categories,
        included_fields,
        sensitive_literal_keys,
        excluded_keys,
    })
}

pub fn parse_settings_import(raw: &str) -> Result<SettingsExportDocument, SettingsImportError> {
    if raw.len() as u64 > SETTINGS_EXPORT_MAX_BYTES {
        return Err(SettingsImportError::TooLarge);
    }
    if !settings_json_looks_like_json(raw.as_bytes()) {
        return Err(SettingsImportError::InvalidJson);
    }
    let value: Value = serde_json::from_str(raw).map_err(|_| SettingsImportError::InvalidJson)?;
    reject_forbidden_value(&value, "")?;
    let format = value
        .get("format")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if format == "bronze-export" || format.is_empty() {
        return Err(SettingsImportError::WrongFormat);
    }
    if format != SETTINGS_EXPORT_FORMAT {
        return Err(SettingsImportError::WrongFormat);
    }
    let version = value.get("version").and_then(Value::as_u64);
    if version != Some(u64::from(SETTINGS_EXPORT_VERSION)) {
        return Err(SettingsImportError::UnknownVersion);
    }
    let parsed: SettingsExportDocument =
        serde_json::from_value(value).map_err(|_| SettingsImportError::InvalidSchema)?;
    parsed
        .settings
        .validate()
        .map_err(|_| SettingsImportError::InvalidSchema)?;
    if let Some(profiles) = &parsed.profiles {
        for profile in profiles {
            if !settings_profile_id_ok(&profile.id) {
                return Err(SettingsImportError::ForbiddenContent);
            }
            if value_looks_like_machine_path(&profile.name) {
                return Err(SettingsImportError::ForbiddenContent);
            }
        }
    }
    Ok(parsed)
}

pub fn export_flags_sensitive_key(key: &str) -> bool {
    key == "privacy.excludedBundleIds"
        || key == "privacy.appPolicies"
        || key == "capture.standardChord"
        || key == "copy.defaultProfileId"
        || key.starts_with("shortcuts.")
        || key.starts_with("profiles.")
}

fn strip_ephemeral_shortcut(mut binding: ShortcutBinding) -> ShortcutBinding {
    binding.tested = None;
    binding
}

fn settings_profile_exportable(
    profile: &SettingsProfileExport,
    excluded_keys: &mut Vec<String>,
) -> bool {
    if !settings_profile_id_ok(&profile.id) {
        excluded_keys.push(format!("profiles.{}", profile.id));
        return false;
    }
    if value_looks_like_machine_path(&profile.name) {
        excluded_keys.push(format!("profiles.{}.name", profile.id));
        return false;
    }
    true
}

fn settings_profile_id_ok(id: &str) -> bool {
    let trimmed = id.trim();
    !trimmed.is_empty()
        && !trimmed.contains('/')
        && !trimmed.contains('\\')
        && !trimmed.contains("..")
}

fn sanitize_object(map: &mut Map<String, Value>, prefix: &str, excluded: &mut Vec<String>) {
    let keys: Vec<String> = map.keys().cloned().collect();
    for key in keys {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        if key == "tested" {
            map.remove(&key);
            excluded.push(path);
            continue;
        }
        if !settings_key_exportable(&key) {
            map.remove(&key);
            excluded.push(path);
            continue;
        }
        match map.get_mut(&key) {
            Some(Value::String(value)) if value_looks_like_machine_path(value) => {
                excluded.push(path);
                *value = String::new();
            }
            Some(Value::Array(items)) => {
                items.retain(|item| match item {
                    Value::String(value) => !value_looks_like_machine_path(value),
                    _ => true,
                });
            }
            Some(Value::Object(child)) => sanitize_object(child, &path, excluded),
            _ => {}
        }
    }
}

fn reject_forbidden_value(value: &Value, prefix: &str) -> Result<(), SettingsImportError> {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if !settings_key_exportable(key) {
                    return Err(SettingsImportError::ForbiddenContent);
                }
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                reject_forbidden_value(child, &path)?;
            }
            Ok(())
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                reject_forbidden_value(child, &format!("{prefix}[{index}]"))?;
            }
            Ok(())
        }
        Value::String(raw) if value_looks_like_machine_path(raw) => {
            Err(SettingsImportError::ForbiddenContent)
        }
        _ => Ok(()),
    }
}

fn collect_sensitive_keys(
    settings: &SettingsV1,
    shortcuts: &[ShortcutBinding],
    profiles: &[SettingsProfileExport],
) -> Vec<String> {
    let mut keys = Vec::new();
    if !settings.privacy.excluded_bundle_ids.is_empty() {
        keys.push("privacy.excludedBundleIds".into());
    }
    if !settings.privacy.app_policies.is_empty() {
        keys.push("privacy.appPolicies".into());
    }
    if !is_default_binding(&settings.capture.standard_chord)
        || (settings.capture.standard_chord.enabled
            && settings.capture.standard_chord.trigger != crate::schema::TriggerKind::Disabled)
    {
        keys.push("capture.standardChord".into());
    }
    if settings.copy.default_profile_id != "plain" {
        keys.push("copy.defaultProfileId".into());
    }
    for binding in shortcuts {
        if !is_default_binding(binding) {
            keys.push(format!("shortcuts.{}", binding.action.as_str()));
        }
    }
    for profile in profiles {
        if !profile.name.trim().is_empty() {
            keys.push(format!("profiles.{}.name", profile.id));
        }
        if let Value::Object(options) = &profile.format_options {
            for (option, value) in options {
                if let Value::String(literal) = value {
                    if !literal.is_empty() && literal != "\n" {
                        keys.push(format!("profiles.{}.{}", profile.id, option));
                    }
                }
            }
        }
    }
    keys.sort();
    keys.dedup();
    keys
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{
        AppPolicy, InheritAllowDeny, InheritAllowDenyAsk, InheritProvenance, SettingsV1,
        ShortcutActionId, TriggerKind,
    };
    use crate::shortcuts::default_shortcut_binding;

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
        assert!(blob.contains("bronze-settings"));
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
        assert!(preview.included_categories.contains(&"privacy".into()));
        assert!(preview
            .included_fields
            .iter()
            .any(|field| field == "general.titleModel"));
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
        settings.copy.default_profile_id = "custom-prompt".into();
        let mut search = default_shortcut_binding(ShortcutActionId::QueueSearch);
        search.logical_key = Some("k".into());
        let profiles = [SettingsProfileExport {
            id: "custom-prompt".into(),
            builtin_key: None,
            name: "Vault prompt".into(),
            format: "promptBlock".into(),
            format_options: serde_json::json!({
                "contextHeading": "Client notes",
                "instructionHeading": "Do this",
                "itemDelimiter": "<<<"
            }),
            source_policy: "none".into(),
            post_copy_action: "copied".into(),
            advance_policy: "keep".into(),
            revision: 1,
        }];
        let preview = export_settings_document(
            &settings,
            &[search],
            &profiles,
            &BTreeMap::new(),
            "2026-09-22T00:00:00Z",
        )
        .expect("export");
        assert!(preview
            .sensitive_literal_keys
            .contains(&"privacy.excludedBundleIds".into()));
        assert!(preview
            .sensitive_literal_keys
            .contains(&"privacy.appPolicies".into()));
        assert!(preview
            .sensitive_literal_keys
            .contains(&"capture.standardChord".into()));
        assert!(preview
            .sensitive_literal_keys
            .contains(&"copy.defaultProfileId".into()));
        assert!(preview
            .sensitive_literal_keys
            .contains(&"shortcuts.queue.search".into()));
        assert!(preview
            .sensitive_literal_keys
            .contains(&"profiles.custom-prompt.name".into()));
        assert!(preview
            .sensitive_literal_keys
            .contains(&"profiles.custom-prompt.contextHeading".into()));
        assert!(export_flags_sensitive_key("privacy.appPolicies"));
        assert!(export_flags_sensitive_key("shortcuts.queue.search"));
        let blob = preview.payload.to_string();
        assert!(blob.contains("com.bank.app"));
        assert!(blob.contains("Vault prompt"));
        assert!(!blob.contains("permissionToken"));
        assert!(!blob.contains("\"tested\""));
    }

    #[test]
    fn settings_import_round_trip_and_rejects_bad_documents() {
        let mut settings = SettingsV1::defaults();
        settings.general.locale = "nl".into();
        settings.general.title_model = crate::schema::TitleModelId::Smol360;
        settings
            .privacy
            .excluded_bundle_ids
            .push("com.bank.app".into());
        let preview = export_settings_document(
            &settings,
            &[],
            &[],
            &BTreeMap::new(),
            "2026-09-22T00:00:00Z",
        )
        .expect("export");
        let raw = serde_json::to_string(&preview.payload).expect("json");
        let parsed = parse_settings_import(&raw).expect("import");
        assert_eq!(parsed.settings.general.locale, "nl");
        assert_eq!(
            parsed.settings.general.title_model,
            crate::schema::TitleModelId::Smol360
        );
        assert_eq!(
            parsed.settings.privacy.excluded_bundle_ids,
            vec!["com.bank.app"]
        );
        assert_eq!(
            parse_settings_import(r#"{"format":"bronze-settings","version":2}"#).unwrap_err(),
            SettingsImportError::UnknownVersion
        );
        assert_eq!(
            parse_settings_import(r#"{"format":"bronze-export","version":1,"settings":{}}"#)
                .unwrap_err(),
            SettingsImportError::WrongFormat
        );
        assert_eq!(
            parse_settings_import(
                r#"{"format":"bronze-settings","version":1,"permissionToken":"abc","settings":{}}"#
            )
            .unwrap_err(),
            SettingsImportError::ForbiddenContent
        );
        assert_eq!(
            parse_settings_import("<script>{}</script>").unwrap_err(),
            SettingsImportError::InvalidJson
        );
        let huge = format!(
            "{{\"format\":\"bronze-settings\"{}}}",
            " ".repeat(2_000_000)
        );
        assert_eq!(
            parse_settings_import(&huge).unwrap_err(),
            SettingsImportError::TooLarge
        );
        assert!(settings_json_looks_like_json(raw.as_bytes()));
        assert!(!settings_json_looks_like_json(b"not-json"));
    }
}
