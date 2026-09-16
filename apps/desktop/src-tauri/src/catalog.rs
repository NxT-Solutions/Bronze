//! Native + WebView catalog parity (story 7.5, I18N-001/002/003/004, G-06).
//! Menu and InfoPlist strings resolve through the same glossary as WebView.

use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

use crate::window_edge::TextDirection;

pub const ADVERTISED_LOCALES: &[&str] = &["en", "en-XA", "ar-XB"];

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

pub fn locales_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../packages/i18n/locales")
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
}
