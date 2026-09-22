mod capture_permissions;
mod live_session;
mod own_selection;

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
                capture_permissions::notification_authorization_status,
                capture_permissions::request_notification_authorization,
                show_chrome_window,
                live_session::list_queue_items,
                live_session::list_overview_items,
                live_session::add_composer_item,
                live_session::apply_queue_item_action,
                live_session::edit_queue_item,
                live_session::copy_queue_items,
                live_session::search_library_items,
                live_session::load_settings_v1,
                live_session::save_settings_v1,
                live_session::reset_settings_field,
                live_session::reset_settings_group,
                live_session::reset_settings_all,
                live_session::search_settings_fields,
                live_session::list_shortcuts,
                live_session::record_shortcut,
                live_session::restore_shortcut,
                live_session::list_installed_apps,
                live_session::pick_installed_app,
                live_session::app_icon_data_url,
                live_session::ui_locale,
                live_session::ui_catalog,
                live_session::backup_library_now,
                live_session::export_library_archive,
                live_session::import_library_archive,
                live_session::preview_settings_export,
                live_session::export_settings_file,
                live_session::import_settings_file,
            ])
            .setup(|app| {
                use tauri::Manager;
                let data_dir = app.path().app_data_dir()?;
                let session = live_session::LiveSession::open(data_dir)?;
                app.manage(std::sync::Mutex::new(session));
                install_chrome_menu(app.handle())?;
                install_status_item(app.handle())?;
                reveal_quick_panel(app.handle())?;
                start_capture_pump(app.handle().clone());
                Ok(())
            })
            .on_menu_event(|app, event| {
                handle_menu_id(app, event.id().as_ref());
            });
    }
    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn show_chrome_window(app: tauri::AppHandle, kind: String) -> Result<(), String> {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
    let label = window_edge::allowed_chrome_window(&kind).ok_or("unknown_window")?;
    let spec = window_edge::chrome_window_spec(label).ok_or("unknown_window")?;
    if let Some(window) = app.get_webview_window(spec.label) {
        window.show().map_err(|err| err.to_string())?;
        let _ = window.set_focus();
        return Ok(());
    }
    let window = WebviewWindowBuilder::new(&app, spec.label, WebviewUrl::App(spec.url.into()))
        .title(spec.title)
        .inner_size(spec.width, spec.height)
        .min_inner_size(spec.min_width, spec.min_height)
        .visible(true)
        .build()
        .map_err(|err| err.to_string())?;
    let _ = window.set_focus();
    Ok(())
}

#[cfg(target_os = "macos")]
fn ui_catalog_map(app: &tauri::AppHandle) -> std::collections::BTreeMap<String, String> {
    use tauri::Manager;
    let locale = app
        .try_state::<std::sync::Mutex<live_session::LiveSession>>()
        .map(|session| {
            session
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .ui_locale()
        })
        .unwrap_or(live_session::HAND_TEST_UI_LOCALE);
    crate::catalog::load_ui_catalog_map(&locales_root(), locale)
}

#[cfg(target_os = "macos")]
fn install_chrome_menu(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
    let map = ui_catalog_map(app);
    let app_name = require_key(&map, "app.name").unwrap_or_else(|_| "Bronze".into());
    let library = require_key(&map, "library.title").unwrap_or_else(|_| "Library".into());
    let settings = require_key(&map, "settings.title").unwrap_or_else(|_| "Settings".into());
    let help = require_key(&map, "help.title").unwrap_or_else(|_| "Help".into());
    let capture = require_key(&map, "menu.status.capture").unwrap_or_else(|_| "Capture".into());
    let show = require_key(&map, "menu.status.show").unwrap_or_else(|_| "Show".into());
    let quit = require_key(&map, "menu.status.quit").unwrap_or_else(|_| "Quit".into());
    let show_library = MenuItem::with_id(app, "show-library", &library, true, None::<&str>)?;
    let show_settings = MenuItem::with_id(app, "show-settings", &settings, true, None::<&str>)?;
    let show_help = MenuItem::with_id(app, "show-help", &help, true, None::<&str>)?;
    let show_panel = MenuItem::with_id(app, "show-panel", &show, true, None::<&str>)?;
    let capture_item = MenuItem::with_id(app, "capture-selection", &capture, true, None::<&str>)?;
    let app_menu = Submenu::with_items(
        app,
        &app_name,
        true,
        &[
            &show_panel,
            &capture_item,
            &show_library,
            &show_settings,
            &show_help,
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
    use tauri::{LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder};
    use window_edge::{
        physical_rect_to_logical, quick_panel_frame, Rect, DEV_LAUNCH_REVEALS_QUICK,
        QUICK_PANEL_HEIGHT, QUICK_PANEL_MIN_HEIGHT, QUICK_PANEL_MIN_WIDTH, QUICK_PANEL_WIDTH,
    };
    if !DEV_LAUNCH_REVEALS_QUICK {
        return Ok(());
    }
    let window = if let Some(window) = app.get_webview_window("quick") {
        window
    } else {
        WebviewWindowBuilder::new(app, "quick", WebviewUrl::App("index.html".into()))
            .title("Bronze")
            .inner_size(f64::from(QUICK_PANEL_WIDTH), f64::from(QUICK_PANEL_HEIGHT))
            .min_inner_size(
                f64::from(QUICK_PANEL_MIN_WIDTH),
                f64::from(QUICK_PANEL_MIN_HEIGHT),
            )
            .visible(true)
            .build()?
    };
    if let Some(monitor) = window.current_monitor()? {
        let area = monitor.work_area();
        let work = physical_rect_to_logical(
            Rect {
                x: area.position.x,
                y: area.position.y,
                width: area.size.width,
                height: area.size.height,
            },
            monitor.scale_factor(),
        );
        let placed = quick_panel_frame(work, catalog_dir("en"));
        window.set_position(LogicalPosition::new(placed.x as f64, placed.y as f64))?;
        window.set_size(LogicalSize::new(
            f64::from(placed.width),
            f64::from(placed.height),
        ))?;
        window.set_min_size(Some(LogicalSize::new(
            f64::from(QUICK_PANEL_MIN_WIDTH),
            f64::from(QUICK_PANEL_MIN_HEIGHT),
        )))?;
    }
    let _ = window.unminimize();
    window.show()?;
    let _ = window.set_focus();
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
            let _ = capture_permissions::prompt_notification_if_undetermined();
            let _ = runtime.event_tap_start();
            // Process-lifetime: dropping would shutdown the in-process static lib.
            std::mem::forget(runtime);
        }
        Err(err) => panic!("BronzeNative init failed; refusing sidecar fallback (ADR-004): {err}"),
    }
}

#[cfg(target_os = "macos")]
const TRAY_ID: &str = "bronze-status";
#[cfg(target_os = "macos")]
const TRAY_COPY_PREFIX: &str = "tray-copy-";
#[cfg(target_os = "macos")]
const TRAY_RECENT_LIMIT: usize = 5;

#[cfg(target_os = "macos")]
fn tray_item_title(body: &str) -> String {
    let flat: String = body
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect();
    let trimmed = flat.trim();
    if trimmed.is_empty() {
        return "…".into();
    }
    let mut title: String = trimmed.chars().take(48).collect();
    if trimmed.chars().count() > 48 {
        title.push('…');
    }
    title
}

fn tray_entry_label(title: Option<&str>, body: &str) -> String {
    let raw = title.filter(|text| !text.is_empty()).unwrap_or(body);
    tray_item_title(raw)
}

#[cfg(target_os = "macos")]
fn status_tray_menu(
    app: &tauri::AppHandle,
) -> Result<tauri::menu::Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, MenuItem};
    use tauri::Manager;
    let map = ui_catalog_map(app);
    let show = require_key(&map, "menu.status.show").unwrap_or_else(|_| "Show".into());
    let capture = require_key(&map, "menu.status.capture").unwrap_or_else(|_| "Capture".into());
    let settings = require_key(&map, "settings.title").unwrap_or_else(|_| "Settings".into());
    let help = require_key(&map, "help.title").unwrap_or_else(|_| "Help".into());
    let quit = require_key(&map, "menu.status.quit").unwrap_or_else(|_| "Quit".into());
    let recent = {
        let session = app.state::<std::sync::Mutex<live_session::LiveSession>>();
        let session = session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        session
            .list_overview()
            .unwrap_or_default()
            .into_iter()
            .take(TRAY_RECENT_LIMIT)
            .collect::<Vec<_>>()
    };
    let mut builder = MenuBuilder::new(app);
    for item in &recent {
        let entry = MenuItem::with_id(
            app,
            format!("{TRAY_COPY_PREFIX}{}", item.id),
            tray_entry_label(item.title.as_deref(), &item.body),
            true,
            None::<&str>,
        )?;
        builder = builder.item(&entry);
    }
    if !recent.is_empty() {
        builder = builder.separator();
    }
    let show_item = MenuItem::with_id(app, "show-panel", &show, true, None::<&str>)?;
    let capture_item = MenuItem::with_id(app, "capture-selection", &capture, true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "show-settings", &settings, true, None::<&str>)?;
    let help_item = MenuItem::with_id(app, "show-help", &help, true, None::<&str>)?;
    builder = builder
        .item(&show_item)
        .item(&capture_item)
        .item(&settings_item)
        .item(&help_item)
        .quit_with_text(&quit);
    Ok(builder.build()?)
}

#[cfg(target_os = "macos")]
fn rebuild_status_menu(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return Ok(());
    };
    // Recents are rebuilt when the menu is popped. Keep the item unbound so
    // AppKit cannot swallow left-click (that click is Show).
    tray.set_menu(None::<tauri::menu::Menu<tauri::Wry>>)?;
    Ok(())
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StatusItemClick {
    ShowPanel,
    PopupMenu,
}

#[cfg(target_os = "macos")]
fn status_item_click(
    button: tauri::tray::MouseButton,
    state: tauri::tray::MouseButtonState,
) -> Option<StatusItemClick> {
    use tauri::tray::{MouseButton, MouseButtonState};
    match button {
        MouseButton::Left => Some(StatusItemClick::ShowPanel),
        MouseButton::Right if state == MouseButtonState::Down => Some(StatusItemClick::PopupMenu),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
static STATUS_APP: std::sync::Mutex<Option<tauri::AppHandle>> = std::sync::Mutex::new(None);

#[cfg(target_os = "macos")]
objc2::define_class!(
    #[unsafe(super(objc2::runtime::NSObject))]
    #[name = "BronzeStatusClickTarget"]
    struct StatusClickTarget;

    impl StatusClickTarget {
        #[unsafe(method(showPanel:))]
        fn show_panel(&self, _sender: Option<&objc2::runtime::AnyObject>) {
            let app = STATUS_APP.lock().ok().and_then(|guard| guard.clone());
            if let Some(app) = app {
                let _ = reveal_quick_panel(&app);
            }
        }
    }
);

#[cfg(target_os = "macos")]
fn remember_status_app(app: &tauri::AppHandle) {
    if let Ok(mut slot) = STATUS_APP.lock() {
        *slot = Some(app.clone());
    }
}

#[cfg(target_os = "macos")]
fn attach_status_button_action(item: &objc2_app_kit::NSStatusItem) {
    use objc2::rc::Retained;
    use objc2::{msg_send, sel, AllocAnyThread};
    use objc2_foundation::MainThreadMarker;
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let Some(button) = item.button(mtm) else {
        return;
    };
    let target: Retained<StatusClickTarget> =
        unsafe { msg_send![StatusClickTarget::alloc(), init] };
    unsafe {
        button.setTarget(Some(&*target));
        button.setAction(Some(sel!(showPanel:)));
    }
    std::mem::forget(target);
}

#[cfg(target_os = "macos")]
fn popup_status_menu(app: &tauri::AppHandle) {
    let Ok(menu) = status_tray_menu(app) else {
        return;
    };
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    // NSStatusItem.performClick places the menu flush under the extra.
    // A WebView context menu is positioned in the panel, not the menu bar.
    let _ = tray.set_menu(Some(menu));
    let _ = tray.with_inner_tray_icon(|inner| inner.show_menu());
    let _ = tray.set_menu(None::<tauri::menu::Menu<tauri::Wry>>);
}

#[cfg(target_os = "macos")]
fn handle_status_item_event(app: &tauri::AppHandle, event: &tauri::tray::TrayIconEvent) {
    use tauri::tray::TrayIconEvent;
    let TrayIconEvent::Click {
        button,
        button_state,
        ..
    } = event
    else {
        return;
    };
    match status_item_click(*button, *button_state) {
        Some(StatusItemClick::ShowPanel) => {
            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                let _ = tray.set_menu(None::<tauri::menu::Menu<tauri::Wry>>);
            }
            let _ = reveal_quick_panel(app);
        }
        Some(StatusItemClick::PopupMenu) => popup_status_menu(app),
        None => {}
    }
}

#[cfg(target_os = "macos")]
fn copy_overview_item(app: &tauri::AppHandle, id: &str) {
    use tauri::Manager;
    let session = app.state::<std::sync::Mutex<live_session::LiveSession>>();
    let mut session = session
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _ = session.copy_items(&[id.to_string()], "plain", &mut live_session::MacPasteboard);
}

#[cfg(target_os = "macos")]
fn start_capture_pump(app: tauri::AppHandle) {
    use bronze_platform_macos::{EventTapHealth, NativeRuntime, BRONZE_TAP_REC_TRIGGER};
    use std::time::{Duration, Instant};
    use tauri::Manager;
    std::thread::Builder::new()
        .name("bronze-capture-pump".into())
        .spawn(move || {
            let mut last_retry = Instant::now()
                .checked_sub(Duration::from_secs(2))
                .unwrap_or_else(Instant::now);
            loop {
                bronze_platform_macos::note_external_focus();
                if last_retry.elapsed() >= Duration::from_secs(1) {
                    let listening = NativeRuntime::event_tap_health_shared()
                        .ok()
                        .is_some_and(|health| health == EventTapHealth::Listening);
                    if !listening {
                        let _ = NativeRuntime::event_tap_start_shared();
                        if let Some(session) =
                            app.try_state::<std::sync::Mutex<live_session::LiveSession>>()
                        {
                            let session = session
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner());
                            session.apply_live_capture_gesture();
                        }
                    }
                    last_retry = Instant::now();
                }
                if let Ok(Some(record)) = NativeRuntime::event_tap_drain_shared() {
                    if record.kind == BRONZE_TAP_REC_TRIGGER {
                        let handle = app.clone();
                        let _ = handle.run_on_main_thread({
                            let handle = handle.clone();
                            move || {
                                on_capture_requested(&handle);
                            }
                        });
                    }
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        })
        .ok();
}

#[cfg(target_os = "macos")]
fn install_status_item(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::tray::TrayIconBuilder;
    let map = ui_catalog_map(app);
    let app_name = require_key(&map, "app.name").unwrap_or_else(|_| "Bronze".into());
    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(&app_name)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            handle_menu_id(app, event.id().as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            handle_status_item_event(tray.app_handle(), &event);
        });
    if let Ok(icon) = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png")) {
        tray = tray.icon(icon).icon_as_template(true);
    } else if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone()).icon_as_template(true);
    }
    remember_status_app(app);
    let tray = tray.build(app)?;
    let _ = tray.with_inner_tray_icon(|inner| {
        if let Some(item) = inner.ns_status_item() {
            attach_status_button_action(&item);
        }
    });
    Ok(())
}

#[cfg(target_os = "macos")]
fn handle_menu_id(app: &tauri::AppHandle, id: &str) {
    if let Some(item_id) = id.strip_prefix(TRAY_COPY_PREFIX) {
        copy_overview_item(app, item_id);
        return;
    }
    match id {
        "show-library" => {
            let _ = show_chrome_window(app.clone(), "library".into());
        }
        "show-settings" => {
            let _ = show_chrome_window(app.clone(), "settings".into());
        }
        "show-help" => {
            let _ = show_chrome_window(app.clone(), "help".into());
        }
        "show-panel" => {
            let _ = reveal_quick_panel(app);
        }
        "capture-selection" => {
            on_capture_requested(app);
        }
        _ => {}
    }
}

#[cfg(target_os = "macos")]
pub fn on_capture_requested(app: &tauri::AppHandle) {
    bronze_platform_macos::note_external_focus();
    let _ = capture_permissions::prompt_on_first_capture_path();
    if bronze_platform_macos::bronze_is_frontmost() {
        let handle = app.clone();
        let _ = std::thread::Builder::new()
            .name("bronze-own-selection".into())
            .spawn(move || {
                let own = own_selection::read_own_webview_selection(&handle);
                let app = handle.clone();
                let _ = handle.run_on_main_thread(move || {
                    persist_capture_request(&app, own);
                });
            });
        return;
    }
    persist_capture_request(app, None);
}

#[cfg(target_os = "macos")]
fn persist_capture_request(app: &tauri::AppHandle, own: Option<own_selection::OwnSelection>) {
    use bronze_capture::FakeAnnouncer;
    use bronze_capture::Terminal;
    use live_session::{CaptureResultDto, LiveCaptureHost};
    use tauri::{Emitter, Manager};
    let visible = app
        .get_webview_window("quick")
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);
    let session_state = app.state::<std::sync::Mutex<live_session::LiveSession>>();
    let mut session = session_state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut announce = FakeAnnouncer::default();
    let catalog = session.ui_catalog();
    let persisted = session.persist_selection(&LiveCaptureHost { own }, &mut announce, visible);
    drop(session);
    if let Ok(outcome) = persisted {
        let saved = outcome.terminal == Terminal::Saved;
        let dto = CaptureResultDto::from_persist(outcome);
        let _ = live_session::deliver_capture_user_notice(&catalog, &dto);
        let _ = app.emit("capture-result", dto);
        if saved {
            let _ = app.emit("queue-changed", ());
        }
    }
    let _ = rebuild_status_menu(app);
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
    catalog_dir, catalog_exists, html_lang, load_locale_map, load_ui_catalog_map, locales_root,
    require_key, ADVERTISED_LOCALES, INFOPLIST_GLOSSARY, NATIVE_GLOSSARY_KEYS, PANEL_CHROME_KEYS,
    SHIPPED_UI_LOCALES, WEBVIEW_GLOSSARY_KEYS,
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

    const CAP_NAMES: [&str; 5] = ["quick", "library", "settings", "onboarding", "help"];
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
                assert!(permissions
                    .iter()
                    .any(|permission| permission.as_str() == Some("allow-settings-live")));
                assert!(permissions.iter().any(|permission| {
                    permission.as_str() == Some("allow-notification-authorization")
                }));
            } else if name == "quick" {
                assert!(permissions
                    .iter()
                    .any(|permission| permission.as_str() == Some("core:default")));
                assert!(permissions
                    .iter()
                    .any(|permission| { permission.as_str() == Some("allow-show-chrome-window") }));
                assert!(permissions
                    .iter()
                    .any(|permission| permission.as_str() == Some("allow-queue-live")));
                let used =
                    fs::read_to_string(manifest_dir().join("permissions/used-permissions.toml"))
                        .expect("permissions");
                assert!(used.contains("list_overview_items"));
            } else if name == "library" {
                assert!(permissions
                    .iter()
                    .any(|permission| permission.as_str() == Some("allow-library-live")));
            } else if name == "help" {
                assert_eq!(
                    permissions,
                    &vec![
                        Value::String("core:default".into()),
                        Value::String("allow-ui-catalog".into())
                    ]
                );
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
        assert_eq!(
            quick["width"].as_u64(),
            Some(u64::from(crate::window_edge::QUICK_PANEL_WIDTH))
        );
        assert_eq!(
            quick["height"].as_u64(),
            Some(u64::from(crate::window_edge::QUICK_PANEL_HEIGHT))
        );
        assert_eq!(
            quick["minWidth"].as_u64(),
            Some(u64::from(crate::window_edge::QUICK_PANEL_MIN_WIDTH))
        );
        assert_eq!(
            quick["minHeight"].as_u64(),
            Some(u64::from(crate::window_edge::QUICK_PANEL_MIN_HEIGHT))
        );
        let reveal = include_str!("lib.rs");
        assert!(reveal.contains("LogicalSize"));
        assert!(reveal.contains("physical_rect_to_logical"));
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
        let help = windows
            .iter()
            .find(|window| window["label"] == "help")
            .expect("help window");
        assert_eq!(help["visible"], false);
        assert_eq!(help["url"], "help.html");
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

    #[test]
    fn live_status_item_and_capture_do_not_steal_focus() {
        assert!(crate::live_session::AX_CAPTURE_LIVE);
        assert!(!crate::window_edge::CAPTURE_ONLY_REVEALS_PANEL);
        use bronze_platform_macos::use_system_focused_fallback;
        let lib = include_str!("lib.rs");
        assert!(lib.contains("TrayIconBuilder"));
        assert!(lib.contains("WebviewWindowBuilder"));
        assert!(lib.contains(r#"WebviewWindowBuilder::new(app, "quick""#));
        assert!(lib.contains("list_overview_items"));
        assert!(lib.contains("list_shortcuts"));
        assert!(lib.contains("record_shortcut"));
        assert!(lib.contains("restore_shortcut"));
        assert!(lib.contains("menu.status.capture"));
        assert!(lib.contains("help.title"));
        assert!(lib.contains("apply_live_capture_gesture"));
        assert!(lib.contains("event_tap_drain_shared"));
        assert!(lib.contains("event_tap_start_shared"));
        let pump = lib.split("fn start_capture_pump").nth(1).expect("pump");
        let pump_end = pump.find("\nfn ").unwrap_or(pump.len());
        assert!(pump[..pump_end].contains("apply_live_capture_gesture"));
        assert!(!pump[..pump_end].contains("event_tap_set_enabled_shared(true)"));
        assert!(lib.contains("prompt_notification_if_undetermined"));
        assert!(lib.contains("note_external_focus"));
        assert!(lib.contains("capture-result"));
        assert!(lib.contains("on_menu_event"));
        assert!(lib.contains("on_tray_icon_event"));
        assert!(lib.contains("show_menu_on_left_click(false)"));
        assert!(lib.contains("menu.status.show"));
        assert_eq!(
            crate::status_item_click(
                tauri::tray::MouseButton::Left,
                tauri::tray::MouseButtonState::Down
            ),
            Some(crate::StatusItemClick::ShowPanel)
        );
        assert_eq!(
            crate::status_item_click(
                tauri::tray::MouseButton::Right,
                tauri::tray::MouseButtonState::Down
            ),
            Some(crate::StatusItemClick::PopupMenu)
        );
        assert_eq!(
            crate::status_item_click(
                tauri::tray::MouseButton::Left,
                tauri::tray::MouseButtonState::Up
            ),
            Some(crate::StatusItemClick::ShowPanel)
        );
        assert!(lib.contains("setAction"));
        assert!(lib.contains("showPanel:"));
        let popup = lib.split("fn popup_status_menu").nth(1).expect("popup");
        let popup_end = popup.find("\nfn ").unwrap_or(popup.len());
        assert!(popup[..popup_end].contains("show_menu"));
        assert!(!popup[..popup_end].contains("popup_menu_at"));
        assert!(lib.contains("queue-changed"));
        let prod = lib.split("#[cfg(test)]").next().expect("prod");
        assert!(!prod.contains("native_item_title"));
        assert!(!prod.contains("spawn_title_refine"));
        assert!(!prod.contains("bronze-item-title"));
        assert!(!prod.contains("set_item_title"));
        assert!(lib.contains("tray_entry_label"));
        let pump = lib.split("fn start_capture_pump").nth(1).expect("pump");
        let pump_end = pump.find("\nfn ").unwrap_or(pump.len());
        assert!(!pump[..pump_end].contains("native_item_title"));
        let persist = lib
            .split("fn persist_capture_request")
            .nth(1)
            .expect("persist");
        let persist_end = persist.find("\nmod ").unwrap_or(persist.len());
        assert!(!persist[..persist_end].contains("native_item_title"));
        assert!(!persist[..persist_end].contains("spawn_title_refine"));
        assert!(!persist[..persist_end].contains("set_item_title"));
        let composer = include_str!("live_session.rs");
        let session_prod = composer
            .split("mod live_session_tests")
            .next()
            .expect("session prod");
        assert!(!session_prod.contains("spawn_title_refine"));
        assert!(!session_prod.contains("set_item_title"));
        assert!(!session_prod.contains("native_item_title"));
        let add = composer
            .split("pub fn add_composer_item")
            .nth(1)
            .expect("composer add");
        let add_end = add.find("\n#[cfg").unwrap_or(add.len());
        assert!(!add[..add_end].contains("native_item_title"));
        assert!(!add[..add_end].contains("spawn_title_refine"));
        let capture_fn = lib
            .split("pub fn on_capture_requested")
            .nth(1)
            .expect("capture fn");
        let prompt_at = capture_fn
            .find("prompt_on_first_capture_path")
            .expect("prompt");
        let snapshot_at = capture_fn.find("note_external_focus").expect("snapshot");
        assert!(
            snapshot_at < prompt_at,
            "frontmost PID must be snapshotted before a permission prompt"
        );
        assert!(capture_fn.contains("read_own_webview_selection"));
        assert!(capture_fn.contains("bronze_is_frontmost"));
        assert!(capture_fn.contains("persist_capture_request"));
        assert!(lib.contains("fn persist_capture_request"));
        assert!(lib.contains("LiveCaptureHost"));
        assert!(
            lib.contains("eval_with_callback")
                || include_str!("own_selection.rs").contains("eval_with_callback")
        );
        assert!(!pump[..pump_end].contains("eval_with_callback"));
        assert!(!pump[..pump_end].contains("read_own_webview_selection"));
        assert!(include_str!("own_selection.rs").contains("OWN_SELECTION_JS"));
        assert!(include_str!("own_selection.rs").contains("password"));
        assert!(!include_str!(
            "../../../../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        )
        .contains("eval_with_callback"));
        assert!(
            capture_fn.contains("deliver_capture_user_notice")
                || lib.contains("deliver_capture_user_notice")
        );
        assert!(lib.contains("capture-result"));
        assert!(!use_system_focused_fallback(Some(42)));
        assert!(use_system_focused_fallback(None));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn tray_prefers_nonempty_title_over_body() {
        assert_eq!(
            crate::tray_entry_label(Some("Headline"), "secret-body-should-not-lead"),
            crate::tray_item_title("Headline")
        );
        assert_eq!(
            crate::tray_entry_label(None, "plain body"),
            crate::tray_item_title("plain body")
        );
        assert_eq!(
            crate::tray_entry_label(Some(""), "plain body"),
            crate::tray_item_title("plain body")
        );
        assert!(
            !crate::tray_entry_label(Some("Headline"), "secret-body-should-not-lead")
                .contains("secret")
        );
        let build = include_str!("../build.rs");
        assert!(!build.contains("FoundationModels"));
        assert!(!build.contains("NaturalLanguage"));
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
