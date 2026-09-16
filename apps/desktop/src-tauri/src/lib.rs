#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "macos")]
    start_native_or_die();

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "macos")]
fn start_native_or_die() {
    match bronze_platform_macos::NativeRuntime::start() {
        Ok(runtime) => {
            let version = runtime
                .abi_version()
                .expect("BronzeNative version query (CAP-004)");
            bronze_platform_macos::check_abi_version(version)
                .expect("BronzeNative ABI mismatch is fail-closed (CAP-004)");
            let _ = runtime.event_tap_start();
            // Process-lifetime: dropping would shutdown the in-process static lib.
            std::mem::forget(runtime);
        }
        Err(err) => panic!("BronzeNative init failed; refusing sidecar fallback (ADR-004): {err}"),
    }
}

mod capabilities;
pub use capabilities::{window_allows, LibraryViewState, WindowCommand, WindowKind};
mod copy;
pub use copy::{copy_items, CopyError, FakePasteboard, Pasteboard, SYNTHETIC_PASTE};
mod window_edge;
pub use window_edge::{
    clamp_to_work_area, parse_physical_edge, place_on_physical_edge, stored_edge_ignores_direction,
    EdgeError, PanelKind, PhysicalEdge, Rect, TextDirection, FOCUS_TRAP, PANEL_KIND,
};

#[cfg(test)]
mod abi_ownership;

#[cfg(test)]
mod event_tap;

#[cfg(test)]
mod ingress;

#[cfg(test)]
pub(crate) fn lock_native_runtime() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use std::fs;
    use std::path::Path;

    const CAP_NAMES: [&str; 4] = ["quick", "library", "settings", "onboarding"];
    const FORBIDDEN: [&str; 4] = ["shell", "fs", "http", "sql"];

    fn manifest_dir() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
    }

    fn read_json(path: &Path) -> Value {
        let raw =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
    }

    #[test]
    fn forbidden_command_from_fixture_window_fails_closed() {
        let caps_dir = manifest_dir().join("capabilities");
        let listed: Vec<_> = fs::read_dir(&caps_dir)
            .unwrap_or_else(|e| panic!("capabilities dir: {e}"))
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(listed.len(), CAP_NAMES.len());

        for name in CAP_NAMES {
            let cap = read_json(&caps_dir.join(format!("{name}.json")));
            assert_eq!(cap["identifier"], name);
            let windows = cap["windows"].as_array().expect("windows array");
            assert_eq!(windows, &vec![Value::String(name.to_string())]);
            let permissions = cap["permissions"].as_array().expect("permissions array");
            assert_eq!(permissions, &vec![Value::String("core:default".into())]);
            for permission in permissions {
                let permission = permission.as_str().expect("permission string");
                for needle in FORBIDDEN {
                    assert!(
                        !permission.to_ascii_lowercase().contains(needle),
                        "{name} grants forbidden permission {permission}"
                    );
                }
            }
        }

        let cargo = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
        for plugin in [
            "tauri-plugin-shell",
            "tauri-plugin-fs",
            "tauri-plugin-http",
            "tauri-plugin-sql",
        ] {
            assert!(!cargo.contains(plugin), "Cargo.toml lists {plugin}");
        }

        let conf = read_json(&manifest_dir().join("tauri.conf.json"));
        let windows = conf["app"]["windows"].as_array().expect("windows array");
        assert!(!windows.is_empty(), "tauri.conf.json has no windows");
        for window in windows {
            assert_eq!(window["devtools"], false);
        }
        let csp = &conf["app"]["security"]["csp"];
        assert_eq!(csp["frame-src"], "'none'");
        assert_eq!(csp["object-src"], "'none'");
        let csp_blob = csp.to_string();
        assert!(!csp_blob.contains("unsafe-eval"));
        const ALLOWED_CSP: &[&str] = &[
            "'self'",
            "'none'",
            "'unsafe-inline'",
            "asset:",
            "ipc:",
            "http://asset.localhost",
            "http://ipc.localhost",
            "data:",
            "blob:",
        ];
        for (directive, value) in csp.as_object().expect("csp object") {
            for token in value.as_str().expect("csp directive").split_whitespace() {
                let lower = token.to_ascii_lowercase();
                assert!(
                    ALLOWED_CSP.contains(&lower.as_str()),
                    "{directive} contains remote origin {token}"
                );
            }
        }
        let capabilities = conf["app"]["security"]["capabilities"]
            .as_array()
            .expect("capabilities list");
        let expected: Vec<Value> = CAP_NAMES
            .iter()
            .map(|n| Value::String((*n).into()))
            .collect();
        assert_eq!(capabilities, &expected);
    }

    #[test]
    fn release_like_bundle_has_no_helper_executable() {
        let conf = read_json(&manifest_dir().join("tauri.conf.json"));
        assert!(
            conf["bundle"].get("externalBin").is_none(),
            "bundle.externalBin would be a helper/sidecar (ADR-004)"
        );
        let cargo = fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
        assert!(
            !cargo.to_ascii_lowercase().contains("sidecar"),
            "desktop crate must not declare a sidecar"
        );
        assert!(
            !cargo.contains("[[bin]]"),
            "extra bin target would be a second TCC subject"
        );
        assert!(
            cargo.contains("default-features = false"),
            "bronze-platform-macos must disable abi-stub when linking BronzeNative"
        );
        assert!(
            !manifest_dir().join("src/helper.rs").exists(),
            "helper.rs must not exist"
        );
    }

    #[cfg(all(target_os = "macos", bronze_native_linked))]
    #[test]
    fn startup_abi_version_check_uses_linked_native() {
        let _guard = crate::lock_native_runtime();
        let runtime = bronze_platform_macos::NativeRuntime::start()
            .expect("linked BronzeNative init (story 2.3)");
        let version = runtime.abi_version().expect("version");
        bronze_platform_macos::check_abi_version(version).expect("fail-closed version");
        assert_eq!(version, bronze_platform_macos::BRONZE_ABI_VERSION);
        runtime.shutdown().expect("shutdown");
    }
}
