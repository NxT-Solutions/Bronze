use crate::abi::{BronzeNativeUtf8View, BRONZE_STATUS_CANCELLED, BRONZE_STATUS_OK};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledApp {
    pub bundle_id: String,
    pub name: String,
}

pub fn try_list_installed_apps() -> Option<Vec<InstalledApp>> {
    native_list_installed_apps_payload().map(|payload| parse_app_list_payload(&payload))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PickInstalledApp {
    Picked(InstalledApp),
    Cancelled,
    Unavailable,
    Invalid,
}

pub fn try_pick_installed_app() -> PickInstalledApp {
    match native_pick_installed_app_payload() {
        NativePick::Payload(payload) => parse_app_list_payload(&payload)
            .into_iter()
            .next()
            .map(PickInstalledApp::Picked)
            .unwrap_or(PickInstalledApp::Invalid),
        NativePick::Cancelled => PickInstalledApp::Cancelled,
        NativePick::Unavailable => PickInstalledApp::Unavailable,
    }
}

pub fn installed_app_from_bundle_path(path: &Path) -> Option<InstalledApp> {
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return None;
    }
    let is_app = path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("app"));
    if !is_app {
        return None;
    }
    read_app_bundle(path)
}

pub fn default_application_roots() -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
        PathBuf::from("/System/Applications/Utilities"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        roots.push(PathBuf::from(home).join("Applications"));
    }
    roots
}

pub fn collect_apps_from_roots(roots: &[PathBuf]) -> Vec<InstalledApp> {
    let mut apps = Vec::new();
    let mut seen = Vec::new();
    for root in roots {
        let Ok(entries) = fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("app"))
            {
                continue;
            }
            let Some(app) = read_app_bundle(&path) else {
                continue;
            };
            let key = app.bundle_id.to_ascii_lowercase();
            if seen.iter().any(|id| id == &key) {
                continue;
            }
            seen.push(key);
            apps.push(app);
        }
    }
    apps.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
            .then_with(|| left.bundle_id.cmp(&right.bundle_id))
    });
    apps
}

pub fn parse_app_list_payload(payload: &str) -> Vec<InstalledApp> {
    let mut apps = Vec::new();
    let mut seen = Vec::new();
    for line in payload.lines() {
        let Some((bundle_id, name)) = line.split_once('\t') else {
            continue;
        };
        let Some(app) = sanitize_installed_app(bundle_id, name) else {
            continue;
        };
        let key = app.bundle_id.to_ascii_lowercase();
        if seen.iter().any(|id| id == &key) {
            continue;
        }
        seen.push(key);
        apps.push(app);
    }
    apps.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
            .then_with(|| left.bundle_id.cmp(&right.bundle_id))
    });
    apps
}

pub fn is_safe_bundle_id(bundle_id: &str) -> bool {
    let id = bundle_id.trim();
    !id.is_empty()
        && id.len() <= 256
        && !id.contains('/')
        && !id.contains('\\')
        && !id.contains('\0')
        && !id.contains("..")
}

fn sanitize_installed_app(bundle_id: &str, name: &str) -> Option<InstalledApp> {
    if !is_safe_bundle_id(bundle_id) {
        return None;
    }
    let name = name.trim().replace(['\t', '\n', '\r'], " ");
    if name.is_empty() || name.starts_with('/') || name.starts_with('~') || name.contains('\\') {
        return None;
    }
    Some(InstalledApp {
        bundle_id: bundle_id.trim().to_string(),
        name,
    })
}

fn read_app_bundle(app_path: &Path) -> Option<InstalledApp> {
    let raw = fs::read_to_string(app_path.join("Contents").join("Info.plist")).ok()?;
    let bundle_id = plist_xml_string(&raw, "CFBundleIdentifier")?;
    let name = plist_xml_string(&raw, "CFBundleDisplayName")
        .or_else(|| plist_xml_string(&raw, "CFBundleName"))
        .or_else(|| {
            app_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_string)
        })?;
    sanitize_installed_app(&bundle_id, &name)
}

fn plist_xml_string(plist: &str, key: &str) -> Option<String> {
    let needle = format!("<key>{key}</key>");
    let after = plist.split_once(&needle)?.1;
    let after = after.trim_start();
    let value = after.strip_prefix("<string>")?.split_once("</string>")?.0;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn native_list_installed_apps_payload() -> Option<String> {
    let mut out = BronzeNativeUtf8View {
        ptr: std::ptr::null(),
        len: 0,
    };
    let status = unsafe { crate::abi::bronze_native_list_installed_apps(&mut out) };
    if status != BRONZE_STATUS_OK {
        return None;
    }
    take_owned_utf8(out)
}

enum NativePick {
    Payload(String),
    Cancelled,
    Unavailable,
}

fn native_pick_installed_app_payload() -> NativePick {
    let mut out = BronzeNativeUtf8View {
        ptr: std::ptr::null(),
        len: 0,
    };
    let status = unsafe { crate::abi::bronze_native_pick_installed_app(&mut out) };
    if status == BRONZE_STATUS_CANCELLED {
        return NativePick::Cancelled;
    }
    if status != BRONZE_STATUS_OK {
        return NativePick::Unavailable;
    }
    take_owned_utf8(out)
        .map(NativePick::Payload)
        .unwrap_or(NativePick::Unavailable)
}

fn take_owned_utf8(out: BronzeNativeUtf8View) -> Option<String> {
    let copied = if out.ptr.is_null() {
        (out.len == 0).then(String::new)
    } else {
        let slice = unsafe { std::slice::from_raw_parts(out.ptr, out.len as usize) };
        String::from_utf8(slice.to_vec()).ok()
    };
    let _ = unsafe { crate::abi::bronze_native_utf8_free(out) };
    copied
}

#[cfg(test)]
mod installed_apps_tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_app(root: &Path, stem: &str, identifier: &str, display: &str) {
        let app = root.join(format!("{stem}.app"));
        let contents = app.join("Contents");
        fs::create_dir_all(&contents).expect("app contents");
        fs::write(
            contents.join("Info.plist"),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key>
  <string>{identifier}</string>
  <key>CFBundleDisplayName</key>
  <string>{display}</string>
</dict>
</plist>
"#
            ),
        )
        .expect("plist");
    }

    #[test]
    fn payload_and_roots_drop_paths_and_keep_many_apps() {
        let payload = [
            "com.apple.Safari\tSafari",
            "/Applications/Evil.app\tEvil",
            "com.ok.one\t/Applications/Foo.app",
            "com.ok.two\tNotes",
            "com.ok.two\tNotes Duplicate",
        ]
        .join("\n");
        let parsed = parse_app_list_payload(&payload);
        assert_eq!(
            parsed,
            vec![
                InstalledApp {
                    bundle_id: "com.ok.two".into(),
                    name: "Notes".into(),
                },
                InstalledApp {
                    bundle_id: "com.apple.Safari".into(),
                    name: "Safari".into(),
                },
            ]
        );
        assert!(parsed.iter().all(|app| !app.bundle_id.contains('/')));
        assert!(parsed.iter().all(|app| !app.name.starts_with('/')));

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("bronze-installed-apps-{stamp}"));
        fs::create_dir_all(&root).expect("root");
        write_app(&root, "Zebra", "com.example.zebra", "Zebra");
        write_app(&root, "Alpha", "com.example.alpha", "Alpha");
        let collected = collect_apps_from_roots(&[root.clone()]);
        let picked = installed_app_from_bundle_path(&root.join("Alpha.app"));
        assert!(installed_app_from_bundle_path(&root.join("Alpha.txt")).is_none());
        fs::remove_dir_all(&root).expect("cleanup");
        assert_eq!(
            collected
                .iter()
                .map(|app| app.bundle_id.as_str())
                .collect::<Vec<_>>(),
            vec!["com.example.alpha", "com.example.zebra"]
        );
        assert!(try_list_installed_apps().is_none());
        assert_eq!(try_pick_installed_app(), PickInstalledApp::Unavailable);
        assert!(!is_safe_bundle_id("../evil"));
        assert!(!is_safe_bundle_id("/Applications/Foo.app"));
        assert!(is_safe_bundle_id("com.apple.Safari"));
        assert_eq!(
            picked,
            Some(InstalledApp {
                bundle_id: "com.example.alpha".into(),
                name: "Alpha".into(),
            })
        );
        assert!(installed_app_from_bundle_path(Path::new("../evil.app")).is_none());
        assert!(picked
            .as_ref()
            .is_some_and(|app| !app.bundle_id.contains('/') && !app.name.contains('/')));
        let swift = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/SourceIconABI.swift"
        ));
        assert!(swift.contains("bronze_native_list_installed_apps"));
        assert!(swift.contains("bronze_native_pick_installed_app"));
        assert!(swift.contains("NSOpenPanel"));
        assert!(swift.contains("bronzeOnAppKitModal"));
        assert!(swift.contains("allowedFileTypes"));
        assert!(swift.contains("/Applications"));
        assert!(!swift.contains("URLSession"));
        assert!(!swift.contains("http://"));
        assert!(!swift.contains("https://"));
        let hop = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/BronzeNative.swift"
        ));
        assert!(hop.contains("bronzeOnAppKitModal"));
        assert!(hop.contains("@MainActor @Sendable"));
        assert!(hop.contains("MainActor.assumeIsolated"));
        assert!(hop.contains("lock.wait()"));
        let tap = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        ));
        assert!(!tap.contains("bronze_native_list_installed_apps"));
        assert!(!tap.contains("bronze_native_pick_installed_app"));
        assert!(!tap.contains("bronze_native_deliver_user_notice"));
        assert!(!tap.contains("bronze_native_request_notification_authorization"));
        assert!(!tap.contains("UNUserNotificationCenter"));
        assert!(!tap.contains("bronze_native_app_icon_png"));
        assert!(!tap.contains("NSOpenPanel"));
    }
}
