//! Native + WebView catalog parity (story 7.5, I18N-001/002/003/004, G-06).
//! Menu and InfoPlist strings resolve through the same glossary as WebView.

use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

use crate::window_edge::TextDirection;

pub const ADVERTISED_LOCALES: &[&str] = &["en", "en-XA", "ar-XB"];

pub const SHIPPED_UI_LOCALES: &[&str] = &["en", "nl", "fr", "de", "es", "it", "en-XA", "ar-XB"];

pub fn html_lang(locale: &str) -> &'static str {
    match locale {
        "nl" => "nl",
        "fr" => "fr",
        "de" => "de",
        "es" => "es",
        "it" => "it",
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

/// Prefer `Contents/Resources/locales` beside the executable.
/// The compile-time checkout exists for `tauri dev` and tests; an installed
/// copy does not have that directory.
pub fn locales_root() -> std::path::PathBuf {
    resolve_locales_root(
        std::env::current_exe().ok().as_deref(),
        &compile_locales_root(),
    )
}

fn compile_locales_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../packages/i18n/locales")
}

fn resolve_locales_root(exe: Option<&Path>, compile_root: &Path) -> std::path::PathBuf {
    if let Some(exe) = exe {
        if let Some(bundled) = bundled_locales_dir(exe) {
            if catalog_exists(&bundled, "en") {
                return bundled;
            }
        }
    }
    compile_root.to_path_buf()
}

fn bundled_locales_dir(exe: &Path) -> Option<std::path::PathBuf> {
    Some(exe.parent()?.join("../Resources/locales"))
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
            ["en", "nl", "fr", "de", "es", "it", "en-XA", "ar-XB"]
        );
        assert!(SHIPPED_UI_LOCALES.contains(&"nl"));
        assert!(!SHIPPED_UI_LOCALES.contains(&"system"));
        assert_eq!(html_lang("nl"), "nl");
        assert_eq!(html_lang("ar-XB"), "ar");
        assert_eq!(html_lang("en-XA"), "en");
        assert_eq!(html_lang("zz"), "en");
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
    fn installed_app_reads_bundled_locales_not_the_compile_checkout() {
        let tmp = std::env::temp_dir().join(format!(
            "bronze-locales-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let exe = tmp.join("Bronze.app/Contents/MacOS/bronze-desktop");
        let bundled = tmp.join("Bronze.app/Contents/Resources/locales");
        std::fs::create_dir_all(bundled.join("en")).expect("bundled locale dir");
        std::fs::create_dir_all(exe.parent().expect("macos dir")).expect("macos dir");
        std::fs::write(&exe, b"").expect("exe");
        std::fs::write(
            bundled.join("en/app.json"),
            r#"{"app":{"name":"FromBundle"}}"#,
        )
        .expect("catalog");
        let missing_checkout = tmp.join("runner/work/Bronze/packages/i18n/locales");
        let resolved = resolve_locales_root(Some(&exe), &missing_checkout);
        assert_eq!(
            resolved.canonicalize().expect("bundle locales"),
            bundled.canonicalize().expect("bundled dir")
        );
        assert_eq!(
            load_locale_map(&resolved, "en")
                .get("app.name")
                .map(String::as_str),
            Some("FromBundle")
        );

        let loose = tmp.join("target/release/bronze-desktop");
        std::fs::create_dir_all(loose.parent().expect("target dir")).expect("target dir");
        std::fs::write(&loose, b"").expect("loose exe");
        let dev = resolve_locales_root(Some(&loose), &compile_locales_root());
        assert!(catalog_exists(&dev, "en"));
        assert!(dev.join("en/app.json").is_file());

        let empty = tmp.join("Empty.app/Contents/MacOS/bronze-desktop");
        std::fs::create_dir_all(empty.parent().expect("empty macos")).expect("empty macos");
        std::fs::write(&empty, b"").expect("empty exe");
        let fallback = resolve_locales_root(Some(&empty), &compile_locales_root());
        assert!(catalog_exists(&fallback, "en"));

        std::fs::remove_dir_all(&tmp).expect("cleanup");
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
}
