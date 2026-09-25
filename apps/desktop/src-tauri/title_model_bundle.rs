use std::path::Path;

/// Tauri errors when this glob matches nothing.
pub const TITLE_GGUF_RESOURCE: &str = "../../../crates/bronze-title-model/vendor/*.gguf";

/// Release packaging still requires vendored weights. Other profiles do not.
pub fn omit_unvendored_title_gguf(profile: &str, gguf_present: bool) -> bool {
    profile != "release" && !gguf_present
}

pub fn gguf_files_present(vendor: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(vendor) else {
        return false;
    };
    entries.filter_map(|entry| entry.ok()).any(|entry| {
        let path = entry.path();
        path.is_file() && path.extension().is_some_and(|ext| ext == "gguf")
    })
}

/// RFC 7396 null deletes the key from the config Tauri already loaded.
pub fn tauri_config_without_title_gguf(existing: Option<&str>) -> String {
    let mut root = existing
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
        .filter(|value| value.is_object())
        .unwrap_or_else(|| serde_json::json!({}));
    let bundle = object_entry(root.as_object_mut().expect("root object"), "bundle");
    let resources = object_entry(bundle, "resources");
    resources.insert(TITLE_GGUF_RESOURCE.to_string(), serde_json::Value::Null);
    root.to_string()
}

fn object_entry<'a>(
    parent: &'a mut serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> &'a mut serde_json::Map<String, serde_json::Value> {
    let entry = parent.entry(key).or_insert_with(|| serde_json::json!({}));
    if !entry.is_object() {
        *entry = serde_json::json!({});
    }
    entry.as_object_mut().expect("object entry")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_without_gguf_drops_the_resource() {
        assert!(omit_unvendored_title_gguf("debug", false));
        let patched = tauri_config_without_title_gguf(None);
        let value: serde_json::Value = serde_json::from_str(&patched).unwrap();
        assert!(value["bundle"]["resources"][TITLE_GGUF_RESOURCE].is_null());
    }

    #[test]
    fn release_and_present_files_keep_the_glob() {
        assert!(!omit_unvendored_title_gguf("release", false));
        assert!(!omit_unvendored_title_gguf("debug", true));
        assert!(!omit_unvendored_title_gguf("release", true));
    }

    #[test]
    fn patch_keeps_other_tauri_config_keys() {
        let patched = tauri_config_without_title_gguf(Some(
            r#"{"app":{"windows":[]},"bundle":{"active":true,"resources":{"../../../packages/i18n/locales":"locales/"}}}"#,
        ));
        let value: serde_json::Value = serde_json::from_str(&patched).unwrap();
        assert_eq!(value["bundle"]["active"], true);
        assert_eq!(
            value["bundle"]["resources"]["../../../packages/i18n/locales"],
            "locales/"
        );
        assert!(value["bundle"]["resources"][TITLE_GGUF_RESOURCE].is_null());
        assert!(value["app"]["windows"].is_array());
    }

    #[test]
    fn gguf_presence_is_top_level_files_only() {
        let dir = std::env::temp_dir().join(format!("bronze-gguf-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("manifests")).unwrap();
        std::fs::write(dir.join("manifests").join("nested.gguf"), b"no").unwrap();
        assert!(!gguf_files_present(&dir));
        assert!(!gguf_files_present(Path::new("/no/such/bronze-vendor")));
        std::fs::write(dir.join("model.gguf"), b"GGUF").unwrap();
        assert!(gguf_files_present(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
