mod capture_permissions;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "macos")]
    start_native_or_die();

    #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
    let mut builder = tauri::Builder::default();
    #[cfg(target_os = "macos")]
    {
        builder = builder
            .invoke_handler(tauri::generate_handler![
                capture_permissions::retest_used_permissions,
                capture_permissions::open_privacy_settings,
                show_chrome_window,
            ])
            .setup(|app| {
                install_chrome_menu(app.handle())?;
                reveal_quick_panel(app.handle())?;
                Ok(())
            })
            .on_menu_event(|app, event| match event.id().as_ref() {
                "show-library" => {
                    let _ = show_chrome_window(app.clone(), "library".into());
                }
                "show-settings" => {
                    let _ = show_chrome_window(app.clone(), "settings".into());
                }
                _ => {}
            });
    }
    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn show_chrome_window(app: tauri::AppHandle, kind: String) -> Result<(), String> {
    use tauri::Manager;
    let label = window_edge::allowed_chrome_window(&kind).ok_or("unknown_window")?;
    let window = app.get_webview_window(label).ok_or("missing_window")?;
    window.show().map_err(|err| err.to_string())?;
    let _ = window.set_focus();
    Ok(())
}

#[cfg(target_os = "macos")]
fn install_chrome_menu(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
    let map = load_locale_map(&locales_root(), "en");
    let app_name = require_key(&map, "app.name").unwrap_or_else(|_| "Bronze".into());
    let library = require_key(&map, "library.title").unwrap_or_else(|_| "Library".into());
    let settings = require_key(&map, "settings.title").unwrap_or_else(|_| "Settings".into());
    let quit = require_key(&map, "menu.status.quit").unwrap_or_else(|_| "Quit".into());
    let show_library = MenuItem::with_id(app, "show-library", &library, true, None::<&str>)?;
    let show_settings = MenuItem::with_id(app, "show-settings", &settings, true, None::<&str>)?;
    let app_menu = Submenu::with_items(
        app,
        &app_name,
        true,
        &[
            &show_library,
            &show_settings,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, Some(&quit))?,
        ],
    )?;
    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    app.set_menu(Menu::with_items(app, &[&app_menu, &edit_menu])?)?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn reveal_quick_panel(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::{Manager, PhysicalPosition, PhysicalSize};
    use window_edge::{quick_panel_frame, Rect, DEV_LAUNCH_REVEALS_QUICK};
    if !DEV_LAUNCH_REVEALS_QUICK {
        return Ok(());
    }
    let Some(window) = app.get_webview_window("quick") else {
        return Ok(());
    };
    if let Some(monitor) = window.current_monitor()? {
        let area = monitor.work_area();
        let work = Rect {
            x: area.position.x,
            y: area.position.y,
            width: area.size.width,
            height: area.size.height,
        };
        let placed = quick_panel_frame(work, catalog_dir("en"));
        window.set_position(PhysicalPosition::new(placed.x, placed.y))?;
        window.set_size(PhysicalSize::new(placed.width, placed.height))?;
    }
    window.show()?;
    Ok(())
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
            let _ = capture_permissions::prompt_on_native_start();
            let _ = runtime.event_tap_start();
            // Process-lifetime: dropping would shutdown the in-process static lib.
            std::mem::forget(runtime);
        }
        Err(err) => panic!("BronzeNative init failed; refusing sidecar fallback (ADR-004): {err}"),
    }
}

#[cfg(target_os = "macos")]
pub fn on_capture_requested() {
    let _ = capture_permissions::prompt_on_first_capture_path();
}

mod capabilities;
pub use capabilities::{window_allows, LibraryViewState, WindowCommand, WindowKind};
mod portability;
pub use portability::{
    accept_native_path, parse_backup_schedule, preview_restore, queue_export_warning_key,
    PathSource, PortabilityError, RestorePreview, PICKER_OWNER_RUST, WEBVIEW_PATHS_ALLOWED,
};
mod copy;
pub use copy::{copy_items, CopyError, FakePasteboard, Pasteboard, SYNTHETIC_PASTE};
mod window_edge;
pub use window_edge::{
    clamp_to_work_area, parse_physical_edge, place_on_physical_edge, stored_edge_ignores_direction,
    EdgeError, PanelKind, PhysicalEdge, Rect, TextDirection, FOCUS_TRAP, PANEL_KIND,
};
mod catalog;
mod packaging;
pub use catalog::{
    catalog_dir, load_locale_map, locales_root, require_key, ADVERTISED_LOCALES,
    INFOPLIST_GLOSSARY, NATIVE_GLOSSARY_KEYS, PANEL_CHROME_KEYS, WEBVIEW_GLOSSARY_KEYS,
};
pub use packaging::{
    forbids_get_task_allow, recorded_arch, sbom_stub, DG01_INTEL_SUPPORT, GET_TASK_ALLOW_FORBIDDEN,
    PACKAGE_ARCH, PACKAGE_SCRIPT,
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
            assert!(
                permissions
                    .iter()
                    .any(|permission| permission.as_str() == Some("core:default")),
                "{name} must include core:default"
            );
            if name == "settings" {
                assert!(permissions.iter().any(|permission| {
                    permission.as_str() == Some("allow-retest-used-permissions")
                }));
                assert!(permissions.iter().any(|permission| {
                    permission.as_str() == Some("allow-open-privacy-settings")
                }));
            } else if name == "quick" {
                assert!(permissions
                    .iter()
                    .any(|permission| permission.as_str() == Some("core:default")));
                assert!(permissions
                    .iter()
                    .any(|permission| { permission.as_str() == Some("allow-show-chrome-window") }));
            } else {
                assert_eq!(permissions, &vec![Value::String("core:default".into())]);
            }
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
        let quick = windows
            .iter()
            .find(|window| window["label"] == "quick")
            .expect("quick window");
        assert_eq!(quick["visible"], true);
        assert_eq!(quick["url"], "index.html");
        let library = windows
            .iter()
            .find(|window| window["label"] == "library")
            .expect("library window");
        assert_eq!(library["visible"], false);
        assert_eq!(library["url"], "library.html");
        let settings = windows
            .iter()
            .find(|window| window["label"] == "settings")
            .expect("settings window");
        assert_eq!(settings["visible"], false);
        assert_eq!(settings["url"], "settings.html");
        assert_eq!(conf["app"]["withGlobalTauri"], true);
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
