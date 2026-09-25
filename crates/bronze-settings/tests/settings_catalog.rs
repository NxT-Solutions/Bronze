//! Settings command DTO joined to the shipped UI catalog.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use bronze_settings::{
    health_rows, search_settings, PermissionSnapshot, PermissionState, SchemaError, SettingsV1,
};
use serde_json::Value;

fn locales_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/i18n/locales")
}

fn flatten(prefix: &str, value: &Value, out: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let next = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten(&next, child, out);
            }
        }
        Value::String(text) => {
            out.insert(prefix.to_string(), text.clone());
        }
        _ => {}
    }
}

fn load_catalog(root: &Path, locale: &str) -> BTreeMap<String, String> {
    let path = root.join(locale).join("app.json");
    let raw =
        fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    let value: Value =
        serde_json::from_str(&raw).unwrap_or_else(|err| panic!("parse {}: {err}", path.display()));
    let mut out = BTreeMap::new();
    flatten("", &value, &mut out);
    out
}

fn require_key(map: &BTreeMap<String, String>, key: &str) -> Result<(), &'static str> {
    match map.get(key) {
        Some(value) if !value.is_empty() => Ok(()),
        Some(_) => Err("empty"),
        None => Err("missing"),
    }
}

#[test]
fn settings_dto_round_trip_resolves_catalog_keys() {
    let settings = SettingsV1::defaults();
    let json = settings.to_json().expect("settings json");
    let restored = SettingsV1::from_json(&json).expect("parse settings json");
    restored.validate().expect("defaults stay valid");
    assert_eq!(restored.general.locale, "system");
    assert_eq!(restored.data.backup_schedule, settings.data.backup_schedule);

    let mut rejected = restored;
    rejected.general.locale = "zz".into();
    assert_eq!(rejected.validate(), Err(SchemaError::UnknownLocale));

    let hits = search_settings("backup");
    assert!(hits.iter().any(|field| field.id == "data.backupSchedule"));
    assert!(search_settings("zzzz-no-such-setting").is_empty());

    let root = locales_root();
    let english = load_catalog(&root, "en");
    assert_eq!(require_key(&english, "settings.title"), Ok(()));
    assert_eq!(
        require_key(&english, "settings.field.backupSchedule"),
        Ok(())
    );
    assert_eq!(require_key(&english, "not.a.catalog.key"), Err("missing"));

    let mut blank = BTreeMap::new();
    blank.insert("settings.title".into(), String::new());
    assert_eq!(require_key(&blank, "settings.title"), Err("empty"));

    let snapshot = PermissionSnapshot {
        input_monitoring: PermissionState::Denied,
        accessibility: PermissionState::NotRequested,
        capture_pipeline_self_test: PermissionState::Unknown,
    };
    for locale in ["en", "nl", "fr", "de", "es", "it", "en-XA", "ar-XB"] {
        let catalog = load_catalog(&root, locale);
        assert_eq!(
            require_key(&catalog, "queue.item.complete"),
            Ok(()),
            "{locale}"
        );
        assert_eq!(require_key(&catalog, "settings.title"), Ok(()), "{locale}");
        for row in health_rows(&snapshot) {
            assert_eq!(
                require_key(&catalog, row.why_key),
                Ok(()),
                "{locale} {}",
                row.why_key
            );
            assert_eq!(
                require_key(&catalog, row.retest_key),
                Ok(()),
                "{locale} {}",
                row.retest_key
            );
            assert_eq!(
                require_key(&catalog, row.alternative_key),
                Ok(()),
                "{locale} {}",
                row.alternative_key
            );
        }
    }
}
