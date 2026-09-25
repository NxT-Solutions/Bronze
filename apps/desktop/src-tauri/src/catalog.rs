//! Native + WebView catalog parity (story 7.5, I18N-001/002/003/004, G-06).
//! Menu and InfoPlist strings resolve through the same glossary as WebView.

use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

use crate::window_edge::TextDirection;

pub const ADVERTISED_LOCALES: &[&str] = &["en", "en-XA", "ar-XB"];

pub const SHIPPED_UI_LOCALES: &[&str] = &[
    "en", "nl", "fr", "de", "es", "it", "ru", "uk", "hr", "sl", "da", "sv", "nb", "fi", "tr",
    "en-XA", "ar-XB",
];

pub fn html_lang(locale: &str) -> &'static str {
    match locale {
        "nl" => "nl",
        "fr" => "fr",
        "de" => "de",
        "es" => "es",
        "it" => "it",
        "ru" => "ru",
        "uk" => "uk",
        "hr" => "hr",
        "sl" => "sl",
        "da" => "da",
        "sv" => "sv",
        "nb" => "nb",
        "fi" => "fi",
        "tr" => "tr",
        "ar-XB" => "ar",
        _ => "en",
    }
}

pub fn catalog_exists(locales_root: &Path, locale: &str) -> bool {
    locales_root.join(locale).join("app.json").is_file()
}

pub fn load_ui_catalog_map(locales_root: &Path, locale: &str) -> BTreeMap<String, String> {
    if catalog_exists(locales_root, locale) {
        load_locale_map(locales_root, locale)
    } else {
        load_locale_map(locales_root, "en")
    }
}

/// InfoPlist slots share catalog keys with WebView (no separate .strings files).
pub const INFOPLIST_GLOSSARY: &[(&str, &str)] = &[
    ("CFBundleDisplayName", "app.name"),
    ("CFBundleName", "app.name"),
];

pub const NATIVE_GLOSSARY_KEYS: &[&str] = &[
    "app.name",
    "menu.status.capture",
    "menu.status.newNote",
    "menu.status.show",
    "menu.status.settings",
    "menu.status.quit",
    "panel.quick.title",
];

pub const WEBVIEW_GLOSSARY_KEYS: &[&str] = &[
    "composer.add.label",
    "queue.item.complete",
    "library.title",
    "settings.title",
];

pub const PANEL_CHROME_KEYS: &[&str] = &[
    "panel.quick.title",
    "panel.section.active",
    "composer.add.label",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CatalogError {
    MissingKey,
    EmptyValue,
}

pub fn catalog_dir(locale: &str) -> TextDirection {
    match locale.split('-').next() {
        Some("ar") | Some("he") | Some("fa") | Some("ur") => TextDirection::Rtl,
        _ => TextDirection::Ltr,
    }
}

pub fn load_locale_map(locales_root: &Path, locale: &str) -> BTreeMap<String, String> {
    let path = locales_root.join(locale).join("app.json");
    let raw =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let value: Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
    let mut out = BTreeMap::new();
    flatten("", &value, &mut out);
    out
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
        Value::String(s) => {
            out.insert(prefix.to_string(), s.clone());
        }
        _ => {}
    }
}

pub fn require_key(map: &BTreeMap<String, String>, key: &str) -> Result<String, CatalogError> {
    match map.get(key) {
        Some(value) if !value.is_empty() => Ok(value.clone()),
        Some(_) => Err(CatalogError::EmptyValue),
        None => Err(CatalogError::MissingKey),
    }
}

/// Directory under Tauri's resource dir (`Contents/Resources` on macOS).
pub const BUNDLED_LOCALES_DIR: &str = "locales";

pub fn workspace_locales_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../packages/i18n/locales")
}

pub fn locales_root() -> std::path::PathBuf {
    resolve_locales_root(
        std::env::current_exe().ok().as_deref(),
        &workspace_locales_root(),
    )
}

/// Packaged apps read `Contents/Resources/locales`. `tauri dev` keeps `workspace`.
pub fn resolve_locales_root(exe: Option<&Path>, workspace: &Path) -> std::path::PathBuf {
    if let Some(exe) = exe {
        if let Some(bundled) = bundled_locales_root(exe) {
            return bundled;
        }
    }
    workspace.to_path_buf()
}

fn bundled_locales_root(exe: &Path) -> Option<std::path::PathBuf> {
    let macos = exe.parent()?;
    if macos.file_name()? != "MacOS" {
        return None;
    }
    let contents = macos.parent()?;
    if contents.file_name()? != "Contents" {
        return None;
    }
    let locales = contents.join("Resources").join(BUNDLED_LOCALES_DIR);
    if catalog_exists(&locales, "en") {
        Some(locales)
    } else {
        None
    }
}

#[cfg(test)]
mod catalog_tests {
    use super::*;
    use crate::window_edge::{
        place_on_physical_edge, stored_edge_ignores_direction, PhysicalEdge, Rect,
    };

    fn maps() -> Vec<(&'static str, BTreeMap<String, String>)> {
        let root = locales_root();
        ADVERTISED_LOCALES
            .iter()
            .map(|locale| (*locale, load_locale_map(&root, locale)))
            .collect()
    }

    #[test]
    fn shipped_ui_locales_resolve_html_lang_and_fallback_catalog() {
        assert_eq!(
            SHIPPED_UI_LOCALES,
            [
                "en", "nl", "fr", "de", "es", "it", "ru", "uk", "hr", "sl", "da", "sv", "nb", "fi",
                "tr", "en-XA", "ar-XB",
            ]
        );
        assert!(SHIPPED_UI_LOCALES.contains(&"nl"));
        assert!(SHIPPED_UI_LOCALES.contains(&"nb"));
        assert!(!SHIPPED_UI_LOCALES.contains(&"nn"));
        assert!(!SHIPPED_UI_LOCALES.contains(&"system"));
        assert_eq!(html_lang("nl"), "nl");
        assert_eq!(html_lang("ar-XB"), "ar");
        assert_eq!(html_lang("en-XA"), "en");
        assert_eq!(html_lang("zz"), "en");
        for tag in ["ru", "uk", "hr", "sl", "da", "sv", "nb", "fi", "tr"] {
            assert_eq!(html_lang(tag), tag);
            assert_eq!(catalog_dir(tag), TextDirection::Ltr);
        }
        let root = locales_root();
        assert!(catalog_exists(&root, "en"));
        let loaded = load_ui_catalog_map(&root, "nl");
        assert!(loaded.contains_key("settings.title"));
        if !catalog_exists(&root, "nl") {
            let english = load_locale_map(&root, "en");
            assert_eq!(loaded.get("settings.title"), english.get("settings.title"));
        }
    }

    #[test]
    fn advertised_locales_cover_native_and_webview_keys() {
        for (locale, map) in maps() {
            for key in NATIVE_GLOSSARY_KEYS
                .iter()
                .chain(WEBVIEW_GLOSSARY_KEYS)
                .chain(PANEL_CHROME_KEYS)
            {
                require_key(&map, key)
                    .unwrap_or_else(|err| panic!("{locale} missing glossary key {key}: {err:?}"));
            }
            for (plist, key) in INFOPLIST_GLOSSARY {
                require_key(&map, key)
                    .unwrap_or_else(|err| panic!("{locale} InfoPlist {plist} key {key}: {err:?}"));
            }
        }
    }

    #[test]
    fn rtl_panel_chrome_keeps_physical_edge() {
        assert_eq!(catalog_dir("ar-XB"), TextDirection::Rtl);
        assert_eq!(catalog_dir("en-XA"), TextDirection::Ltr);
        assert_eq!(catalog_dir("en"), TextDirection::Ltr);
        let work = Rect {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        };
        let preferred = Rect {
            x: 0,
            y: 0,
            width: 320,
            height: 480,
        };
        let ltr = place_on_physical_edge(PhysicalEdge::Right, preferred, work, catalog_dir("en"));
        let rtl =
            place_on_physical_edge(PhysicalEdge::Right, preferred, work, catalog_dir("ar-XB"));
        assert_eq!(ltr, rtl);
        assert_eq!(
            stored_edge_ignores_direction(PhysicalEdge::Left, catalog_dir("ar-XB")),
            PhysicalEdge::Left
        );
        let map = load_locale_map(&locales_root(), "ar-XB");
        for key in PANEL_CHROME_KEYS {
            assert!(!require_key(&map, key).expect(key).is_empty());
        }
    }

    #[test]
    fn item_cards_keep_per_item_lang_and_dir() {
        let ar = bronze_domain::Item::new(1, 1, "مرحبا".into(), Some("ar"), 1).unwrap();
        assert_eq!(ar.card_lang(), "ar");
        assert_eq!(ar.card_dir(), "auto");
        let en = bronze_domain::Item::new(2, 1, "hello".into(), Some("en"), 1).unwrap();
        assert_eq!(en.card_lang(), "en");
        assert_eq!(en.card_dir(), "auto");
        assert_ne!(ar.card_lang(), en.card_lang());
    }

    #[test]
    fn packaged_lookup_ignores_build_machine_absolute_path() {
        let root = std::env::temp_dir().join(format!(
            "bronze-catalog-bundle-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let exe = root.join("Bronze.app/Contents/MacOS/bronze-desktop");
        std::fs::create_dir_all(exe.parent().expect("macos dir")).expect("macos dir");
        std::fs::write(&exe, b"").expect("exe");
        let bundled = root.join("Bronze.app/Contents/Resources/locales");
        std::fs::create_dir_all(bundled.join("en")).expect("locales");
        std::fs::write(bundled.join("en/app.json"), r#"{"app":{"name":"Bronze"}}"#).expect("json");

        let runner = Path::new("/Users/runner/work/Bronze/Bronze/packages/i18n/locales");
        let resolved = resolve_locales_root(Some(&exe), runner);
        assert_eq!(resolved, bundled);
        assert_ne!(resolved, runner);
        assert_ne!(resolved, workspace_locales_root());
        assert!(!resolved.starts_with(env!("CARGO_MANIFEST_DIR")));
        let map = load_locale_map(&resolved, "en");
        assert_eq!(map.get("app.name").map(String::as_str), Some("Bronze"));

        let dev_exe = root.join("target/debug/bronze-desktop");
        std::fs::create_dir_all(dev_exe.parent().expect("debug dir")).expect("debug dir");
        std::fs::write(&dev_exe, b"").expect("dev exe");
        let dev_workspace = root.join("workspace-locales");
        std::fs::create_dir_all(dev_workspace.join("en")).expect("dev locales");
        std::fs::write(
            dev_workspace.join("en/app.json"),
            r#"{"app":{"name":"Dev"}}"#,
        )
        .expect("dev json");
        assert_eq!(
            resolve_locales_root(Some(&dev_exe), &dev_workspace),
            dev_workspace
        );

        let conf =
            std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json"))
                .expect("tauri.conf");
        assert!(conf.contains("\"../../../packages/i18n/locales\": \"locales/\""));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_locale_falls_back_and_rejects_empty_or_absent_keys() {
        let root = std::env::temp_dir().join(format!(
            "bronze-catalog-fallback-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("en")).expect("en dir");
        std::fs::write(
            root.join("en/app.json"),
            r#"{"app":{"name":"Bronze","count":1},"blank":"","settings":{"title":"Settings"}}"#,
        )
        .expect("en json");
        assert!(!catalog_exists(&root, "zz"));
        let loaded = load_ui_catalog_map(&root, "zz");
        assert_eq!(loaded.get("app.name").map(String::as_str), Some("Bronze"));
        assert_eq!(
            loaded.get("settings.title").map(String::as_str),
            Some("Settings")
        );
        assert!(!loaded.contains_key("app.count"));
        assert_eq!(require_key(&loaded, "blank"), Err(CatalogError::EmptyValue));
        assert_eq!(
            require_key(&loaded, "menu.status.capture"),
            Err(CatalogError::MissingKey)
        );
        assert_eq!(html_lang("fr"), "fr");
        assert_eq!(html_lang("de"), "de");
        assert_eq!(html_lang("es"), "es");
        assert_eq!(html_lang("it"), "it");
        assert_eq!(catalog_dir("he"), TextDirection::Rtl);
        assert_eq!(catalog_dir("fa-IR"), TextDirection::Rtl);
        assert_eq!(catalog_dir("ur"), TextDirection::Rtl);
        assert_eq!(catalog_dir("en-US"), TextDirection::Ltr);

        let exe = root.join("Bronze.app/Contents/MacOS/bronze-desktop");
        std::fs::create_dir_all(exe.parent().expect("macos")).expect("macos");
        std::fs::write(&exe, b"").expect("exe");
        let bundled = root.join("Bronze.app/Contents/Resources/locales");
        std::fs::create_dir_all(&bundled).expect("empty bundle locales");
        let workspace = root.join("workspace");
        std::fs::create_dir_all(workspace.join("en")).expect("workspace en");
        std::fs::write(
            workspace.join("en/app.json"),
            r#"{"app":{"name":"Workspace"}}"#,
        )
        .expect("workspace json");
        assert_eq!(resolve_locales_root(Some(&exe), &workspace), workspace);
        let _ = std::fs::remove_dir_all(&root);
    }
}
