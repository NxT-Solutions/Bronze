//! Hand-test queue/settings/portability IPC (QUE-002/005/007, SET-001, DAT-002/003).

use crate::catalog::{
    catalog_dir, catalog_exists, html_lang, load_ui_catalog_map, locales_root, SHIPPED_UI_LOCALES,
};
use crate::copy::{copy_items, CopyError, Pasteboard};
use crate::portability::{accept_native_path, PathSource};
use crate::window_edge::{TextDirection, CAPTURE_ONLY_REVEALS_PANEL};
use bronze_capture::{
    apply_capture_success, Announcer, AxOutcome, CaptureCoordinator, CaptureIngressContext,
    CaptureMode, CapturedText, FocusOwner, FocusSnapshot, PersistError, PersistHook, Terminal,
    INGRESS_ROUTE_MENU,
};
#[cfg(test)]
use bronze_capture::{ax_capture, FakeAxTree};
use bronze_diagnostics::{
    redact_home_path, render_support_report, SupportEventLine, SupportNamedState,
    SupportQueueCounts, SupportReport, SupportSourceApp, AUTOMATIC_UPLOAD, LAST_HOUR_MS,
};
use bronze_domain::{
    default_output_profile, ComposerChord, OutputFormat, OutputProfile, PostCopyAction,
};
use bronze_platform_macos::{
    accept_settings_file_path, accept_support_file_path, read_settings_import_bytes,
    try_pick_settings_export_path, try_pick_settings_import_path, try_pick_support_export_path,
    PickSettingsFile, SettingsFileError,
};
#[cfg(not(test))]
use bronze_platform_macos::{snapshot_from_preflight, MacosPreflightHost};
use bronze_settings::{
    apply_recorded_double_tap_timing, default_shortcut_binding, effective_tap_count,
    export_settings_document, is_default_binding, logical_key_allowed, parse_settings_import,
    recorded_double_tap_allowed, search_settings, shortcut_scope, skip_test_marks_untested,
    CaptureAlternatives, Modifier, NativeRegistrar, RegisterError, SettingsExportPreview,
    SettingsGroup, SettingsImportError, SettingsV1, ShortcutActionId, ShortcutBinding,
    ShortcutRegistry, ShortcutScope, TestedState, TitleModelId, TriggerKind, MAX_MODIFIER_TAPS,
    SCHEMA_VERSION, SETTINGS_EXPORT_FORMAT, SETTINGS_EXPORT_VERSION,
};
use bronze_storage::{
    ComposerDraft, DiagnosticEventRow, ImportStrategy, NoopBackup, Overwrite, PathLocator,
    QueueAction, QueueItemRow, Store, ADR_018_STATUS, APP_SCHEMA_VERSION, QUE_007_COMPLETE,
};
use bronze_title_model::{
    auto_pick_title_tier, present_gguf_tiers, request_tier, TitleTier, GGUF_TIERS,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const HAND_TEST_UI_LOCALE: &str = "en";
pub const AX_CAPTURE_LIVE: bool = true;
const _: () = assert!(AX_CAPTURE_LIVE);
const _: () = assert!(!CAPTURE_ONLY_REVEALS_PANEL);
const _: () = assert!(!AUTOMATIC_UPLOAD);

pub fn is_overview_status(status: &str) -> bool {
    matches!(status, "queued" | "copied" | "active")
}

pub trait SelectionHost {
    fn peek_bundle_id(&self) -> Option<String>;
    fn read(&self) -> (AxOutcome, Option<CapturedText>);
}

#[cfg(test)]
pub struct FakeSelectionHost {
    pub tree: FakeAxTree,
    pub source_app_name: Option<String>,
    pub source_bundle_id: Option<String>,
}

#[cfg(test)]
impl SelectionHost for FakeSelectionHost {
    fn peek_bundle_id(&self) -> Option<String> {
        self.source_bundle_id.clone()
    }

    fn read(&self) -> (AxOutcome, Option<CapturedText>) {
        let (outcome, text) = ax_capture(&self.tree);
        (
            outcome,
            text.map(|mut captured| {
                captured.source_app_name = self.source_app_name.clone();
                captured.source_bundle_id = self.source_bundle_id.clone();
                captured
            }),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturePersistOutcome {
    pub terminal: Terminal,
    pub reason: &'static str,
    pub item_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CaptureResultDto {
    pub terminal: String,
    pub reason: String,
}

impl CaptureResultDto {
    pub fn from_persist(outcome: CapturePersistOutcome) -> Self {
        Self {
            terminal: match outcome.terminal {
                Terminal::Saved => "saved",
                Terminal::Rejected => "rejected",
                Terminal::Failed
                | Terminal::TriggerQueueOverflow
                | Terminal::ContextUnavailable => "failed",
                Terminal::Cancelled => "cancelled",
            }
            .into(),
            reason: outcome.reason.into(),
        }
    }
}

pub struct LiveAxHost;

pub struct LiveCaptureHost {
    pub own: Option<crate::own_selection::OwnSelection>,
}

impl SelectionHost for LiveCaptureHost {
    fn peek_bundle_id(&self) -> Option<String> {
        if matches!(
            self.own,
            Some(crate::own_selection::OwnSelection::Text(_))
                | Some(crate::own_selection::OwnSelection::TooLarge)
        ) {
            bronze_platform_macos::native_bundle_id_for_pid(std::process::id() as i32)
        } else {
            LiveAxHost.peek_bundle_id()
        }
    }

    fn read(&self) -> (AxOutcome, Option<CapturedText>) {
        match &self.own {
            Some(crate::own_selection::OwnSelection::Text(text)) => (
                AxOutcome::Captured { len: text.len() },
                Some(CapturedText {
                    text: text.clone(),
                    source_app_name: Some(crate::own_selection::OWN_SELECTION_SOURCE_NAME.into()),
                    source_bundle_id: bronze_platform_macos::native_bundle_id_for_pid(
                        std::process::id() as i32,
                    ),
                }),
            ),
            Some(crate::own_selection::OwnSelection::TooLarge) => {
                (AxOutcome::SelectionTooLarge, None)
            }
            None => LiveAxHost.read(),
        }
    }
}

impl SelectionHost for LiveAxHost {
    fn peek_bundle_id(&self) -> Option<String> {
        if bronze_platform_macos::bronze_is_frontmost() {
            bronze_platform_macos::native_bundle_id_for_pid(std::process::id() as i32)
        } else {
            bronze_platform_macos::last_external_pid()
                .and_then(bronze_platform_macos::native_bundle_id_for_pid)
        }
    }

    fn read(&self) -> (AxOutcome, Option<CapturedText>) {
        let (outcome, text, source_app_name) = bronze_platform_macos::read_capture_selection();
        let mapped = match outcome {
            bronze_platform_macos::LiveAxOutcome::Captured { len } => AxOutcome::Captured { len },
            bronze_platform_macos::LiveAxOutcome::NoSelection => AxOutcome::NoSelection,
            bronze_platform_macos::LiveAxOutcome::ProtectedContent => AxOutcome::ProtectedContent,
            bronze_platform_macos::LiveAxOutcome::ProtectionUnknown => AxOutcome::ProtectionUnknown,
            bronze_platform_macos::LiveAxOutcome::AccessibilityDenied => {
                AxOutcome::AccessibilityDenied
            }
            bronze_platform_macos::LiveAxOutcome::FocusedElementMissing => {
                AxOutcome::FocusedElementMissing
            }
            bronze_platform_macos::LiveAxOutcome::SelectionTooLarge => AxOutcome::SelectionTooLarge,
            bronze_platform_macos::LiveAxOutcome::InvalidTextEncoding => {
                AxOutcome::InvalidTextEncoding
            }
        };
        let source_bundle_id = if source_app_name
            .as_deref()
            .is_some_and(bronze_platform_macos::is_skipped_process_name)
        {
            bronze_platform_macos::native_bundle_id_for_pid(std::process::id() as i32)
        } else {
            bronze_platform_macos::last_external_pid()
                .and_then(bronze_platform_macos::native_bundle_id_for_pid)
        };
        (
            mapped,
            text.map(|body| CapturedText {
                text: body,
                source_app_name,
                source_bundle_id,
            }),
        )
    }
}

struct SessionPersist<'a> {
    session: &'a mut LiveSession,
    body: String,
    source_app_name: Option<String>,
    source_bundle_id: Option<String>,
    saved_id: &'a mut Option<String>,
}

impl PersistHook for SessionPersist<'_> {
    fn persist(&mut self, _request_id: u64) -> Result<(), PersistError> {
        let item = self
            .session
            .add_captured(
                self.body.clone(),
                self.source_app_name.clone(),
                self.source_bundle_id.clone(),
            )
            .map_err(|_| PersistError)?;
        *self.saved_id = Some(item.id);
        Ok(())
    }
}

const _: () = assert!(!QUE_007_COMPLETE);
const _: () = assert!(matches!(ADR_018_STATUS.as_bytes(), b"Proposed"));

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QueueItemDto {
    pub id: String,
    pub section_id: String,
    pub body: String,
    pub title: Option<String>,
    pub content_language: String,
    pub status: String,
    pub rank: String,
    pub source_app_name: Option<String>,
    #[serde(default)]
    pub source_app_icon: Option<String>,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InstalledAppDto {
    pub bundle_id: String,
    pub name: String,
}

impl From<bronze_platform_macos::InstalledApp> for InstalledAppDto {
    fn from(app: bronze_platform_macos::InstalledApp) -> Self {
        Self {
            bundle_id: app.bundle_id,
            name: app.name,
        }
    }
}

impl fmt::Debug for QueueItemDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueueItemDto")
            .field("id", &self.id)
            .field("section_id", &self.section_id)
            .field("content_language", &self.content_language)
            .field("status", &self.status)
            .field("rank", &self.rank)
            .field("source_app_name", &self.source_app_name)
            .field(
                "source_app_icon",
                &self
                    .source_app_icon
                    .as_ref()
                    .map(|_| "data:image/png;base64"),
            )
            .finish_non_exhaustive()
    }
}

impl From<QueueItemRow> for QueueItemDto {
    fn from(row: QueueItemRow) -> Self {
        let source_app_icon = source_app_icon_data_url(
            row.source_bundle_id.as_deref(),
            row.source_app_name.as_deref(),
        );
        Self {
            id: row.id,
            section_id: row.section_id,
            body: row.body,
            title: row.title,
            content_language: row.content_language,
            status: row.status,
            rank: row.rank,
            source_app_name: row.source_app_name,
            source_app_icon,
        }
    }
}

fn source_app_icon_data_url(bundle: Option<&str>, name: Option<&str>) -> Option<String> {
    for key in [bundle, name]
        .into_iter()
        .flatten()
        .filter(|value| !value.is_empty())
    {
        if let Some(png) = bronze_platform_macos::native_app_icon_png(key) {
            if !png.is_empty() {
                return Some(format!("data:image/png;base64,{}", encode_base64(&png)));
            }
        }
    }
    None
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = chunk.get(1).copied().unwrap_or(0);
        let c = chunk.get(2).copied().unwrap_or(0);
        out.push(TABLE[(a >> 2) as usize] as char);
        out.push(TABLE[(((a & 3) << 4) | (b >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(((b & 15) << 2) | (c >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(c & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

pub fn effective_ui_locale(requested: Option<&str>) -> &'static str {
    let raw = requested.unwrap_or("system").trim();
    if raw.is_empty() || raw.eq_ignore_ascii_case("system") {
        return HAND_TEST_UI_LOCALE;
    }
    if let Some(exact) = SHIPPED_UI_LOCALES.iter().copied().find(|tag| *tag == raw) {
        return exact;
    }
    let primary = raw.split(['-', '_']).next().unwrap_or("");
    SHIPPED_UI_LOCALES
        .iter()
        .copied()
        .find(|tag| *tag == primary)
        .unwrap_or(HAND_TEST_UI_LOCALE)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiCatalogDto {
    pub locale: String,
    pub html_lang: String,
    pub dir: String,
    pub catalog_available: bool,
    pub messages: std::collections::BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutRowDto {
    pub action: String,
    pub trigger: String,
    pub modifiers: Vec<String>,
    pub logical_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tap_count: Option<u32>,
    pub enabled: bool,
    pub is_default: bool,
    pub scope: String,
    pub tested: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutRecordInput {
    pub action: String,
    pub trigger: String,
    pub modifiers: Vec<String>,
    pub logical_key: Option<String>,
    #[serde(default)]
    pub tap_count: Option<u32>,
    pub skip_test: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsExportPreviewDto {
    pub format: String,
    pub version: u32,
    pub included_categories: Vec<String>,
    pub included_fields: Vec<String>,
    pub sensitive_literal_keys: Vec<String>,
    pub excluded_keys: Vec<String>,
}

impl SettingsExportPreviewDto {
    fn from_preview(preview: SettingsExportPreview) -> Self {
        Self {
            format: SETTINGS_EXPORT_FORMAT.into(),
            version: SETTINGS_EXPORT_VERSION,
            included_categories: preview.included_categories,
            included_fields: preview.included_fields,
            sensitive_literal_keys: preview.sensitive_literal_keys,
            excluded_keys: preview.excluded_keys,
        }
    }
}

struct SessionRegistrar;

impl NativeRegistrar for SessionRegistrar {
    fn try_register(&mut self, binding: &ShortcutBinding) -> Result<(), RegisterError> {
        if !binding.enabled || binding.trigger == TriggerKind::Disabled {
            return Ok(());
        }
        match binding.trigger {
            TriggerKind::Disabled => Ok(()),
            TriggerKind::Accelerator => logical_key_ok(binding),
            TriggerKind::ModifierDoubleTap => {
                if recorded_double_tap_allowed(binding) {
                    Ok(())
                } else {
                    Err(RegisterError::NativeRejected)
                }
            }
        }
    }
}

fn logical_key_ok(binding: &ShortcutBinding) -> Result<(), RegisterError> {
    match binding.logical_key.as_deref() {
        Some(key) if logical_key_allowed(key) => Ok(()),
        _ => Err(RegisterError::NativeRejected),
    }
}

fn keep_menu_manual() -> CaptureAlternatives {
    CaptureAlternatives {
        chord: true,
        menu: true,
        manual: true,
    }
}

fn register_error_code(err: RegisterError) -> String {
    match err {
        RegisterError::NativeRejected => "shortcut_rejected".into(),
        RegisterError::Duplicate => "shortcut_duplicate".into(),
        RegisterError::AlternativesRequired => "shortcut_alternatives_required".into(),
    }
}

pub fn capture_notice_catalog_key(terminal: &str, reason: &str) -> &'static str {
    if terminal == "saved" {
        return "capture.announce.saved";
    }
    match reason {
        "accessibility" => "capture.announce.denied",
        "protected" => "capture.announce.protected",
        "no_selection" => "capture.announce.rejected",
        "app_excluded" => "capture.announce.excluded",
        _ => "capture.announce.failed",
    }
}

pub fn deliver_capture_user_notice(
    catalog: &UiCatalogDto,
    dto: &CaptureResultDto,
) -> Result<(), String> {
    let key = capture_notice_catalog_key(&dto.terminal, &dto.reason);
    let title = catalog
        .messages
        .get("app.name")
        .or_else(|| catalog.messages.get("panel.quick.title"))
        .map(String::as_str)
        .unwrap_or("Bronze");
    let body = catalog.messages.get(key).map(String::as_str).unwrap_or("");
    bronze_platform_macos::try_deliver_user_notice(title, body)
        .map_err(|_| "notice_unavailable".into())
}

pub fn named_output_profile(name: &str) -> OutputProfile {
    match name {
        "markdown" | "markdown-bullets" => OutputProfile {
            format: OutputFormat::MarkdownBullets {
                marker: '-',
                separator: "\n".into(),
            },
            post_copy_action: PostCopyAction::Copied,
            advance_policy: default_output_profile().advance_policy,
        },
        _ => default_output_profile(),
    }
}

pub struct LiveSession {
    store: Store,
    settings: SettingsV1,
    shortcuts: ShortcutRegistry,
    data_dir: PathBuf,
}

struct TerminalDiag {
    stage: &'static str,
    result: &'static str,
    trigger: &'static str,
    provider: &'static str,
    source_bundle_id: Option<String>,
    store_result: Option<&'static str>,
    duration_ms: Option<i64>,
}

impl LiveSession {
    pub fn open(data_dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&data_dir).map_err(|err| err.to_string())?;
        let locator = PathLocator {
            path: data_dir.join("bronze.sqlite"),
        };
        let mut backup = NoopBackup;
        let mut store = Store::open(&locator, &mut backup).map_err(|_| "store_open_failed")?;
        let mut settings = load_settings_file(&data_dir)?;
        if resolve_title_model(&mut settings) {
            persist_settings_file(&data_dir, &settings)?;
        }
        apply_title_engine(&settings);
        apply_login_item(&settings);
        let shortcuts = store
            .load_shortcuts()
            .map_err(|_| "shortcuts_load_failed")?;
        store
            .persist_shortcuts(&shortcuts)
            .map_err(|_| "shortcuts_persist_failed")?;
        let session = Self {
            store,
            settings,
            shortcuts,
            data_dir,
        };
        session.apply_live_capture_gesture();
        let now = now_ms();
        session
            .store
            .ensure_inbox(now)
            .map_err(|_| "inbox_seed_failed")?;
        Ok(session)
    }

    pub fn apply_live_capture_gesture(&self) {
        bronze_platform_macos::apply_live_shift_gesture(self.shortcuts.standard_chord());
    }

    pub fn ui_locale(&self) -> &'static str {
        effective_ui_locale(Some(self.settings.general.locale.as_str()))
    }

    pub fn ui_catalog(&self) -> UiCatalogDto {
        Self::catalog_for(self.ui_locale())
    }

    fn catalog_for(locale: &'static str) -> UiCatalogDto {
        let root = locales_root();
        let catalog_available = catalog_exists(&root, locale);
        let applied = if catalog_available { locale } else { "en" };
        let dir = match catalog_dir(applied) {
            TextDirection::Rtl => "rtl",
            TextDirection::Ltr => "ltr",
        };
        UiCatalogDto {
            locale: locale.to_string(),
            html_lang: html_lang(applied).to_string(),
            dir: dir.into(),
            catalog_available,
            messages: load_ui_catalog_map(&root, locale),
        }
    }

    pub fn unavailable_catalog() -> UiCatalogDto {
        Self::catalog_for(HAND_TEST_UI_LOCALE)
    }

    pub fn list_queue(&self, include_trashed: bool) -> Result<Vec<QueueItemDto>, String> {
        self.store
            .list_items(include_trashed)
            .map(|rows| rows.into_iter().map(QueueItemDto::from).collect())
            .map_err(|_| "queue_list_failed".into())
    }

    pub fn list_overview(&self) -> Result<Vec<QueueItemDto>, String> {
        Ok(self
            .list_queue(false)?
            .into_iter()
            .filter(|item| is_overview_status(&item.status))
            .collect())
    }

    pub fn item_body(&self, id: &str) -> Option<String> {
        self.store.get_item(id).ok().map(|row| row.body)
    }

    pub fn apply_refined_title(
        &mut self,
        id: &str,
        expected_body: &str,
        title: &str,
    ) -> Result<bool, String> {
        let row = self
            .store
            .get_item(id)
            .map_err(|_| "not_found".to_string())?;
        if row.body != expected_body {
            return Ok(false);
        }
        self.store
            .set_item_title(id, title, now_ms())
            .map_err(|_| "title_store_failed".to_string())?;
        Ok(true)
    }

    pub fn persist_selection(
        &mut self,
        host: &dyn SelectionHost,
        announcer: &mut dyn Announcer,
        webview_visible: bool,
    ) -> Result<CapturePersistOutcome, String> {
        let peeked = host.peek_bundle_id();
        let started = now_ms();
        let outcome = self.persist_selection_apply(host, announcer, webview_visible)?;
        self.record_terminal_diag(TerminalDiag {
            stage: capture_stage(outcome.reason),
            result: capture_result_code(outcome.reason),
            trigger: "menu",
            provider: "ax",
            source_bundle_id: peeked,
            store_result: Some(if outcome.terminal == Terminal::Saved {
                "ok"
            } else {
                "failed"
            }),
            duration_ms: Some(now_ms().saturating_sub(started)),
        });
        Ok(outcome)
    }

    fn persist_selection_apply(
        &mut self,
        host: &dyn SelectionHost,
        announcer: &mut dyn Announcer,
        webview_visible: bool,
    ) -> Result<CapturePersistOutcome, String> {
        if let Some(bundle) = host.peek_bundle_id() {
            if self.settings.privacy.excludes_bundle(&bundle) {
                return Ok(CapturePersistOutcome {
                    terminal: Terminal::Rejected,
                    reason: "app_excluded",
                    item_id: None,
                });
            }
        }
        let (outcome, text) = host.read();
        match (outcome, text) {
            (AxOutcome::Captured { .. }, Some(captured)) => {
                if captured
                    .source_bundle_id
                    .as_deref()
                    .is_some_and(|bundle| self.settings.privacy.excludes_bundle(bundle))
                {
                    return Ok(CapturePersistOutcome {
                        terminal: Terminal::Rejected,
                        reason: "app_excluded",
                        item_id: None,
                    });
                }
                let body = captured.text;
                let mut saved_id = None;
                let mut coordinator = CaptureCoordinator::with_hooks(
                    8,
                    SessionPersist {
                        session: self,
                        body,
                        source_app_name: captured.source_app_name,
                        source_bundle_id: captured.source_bundle_id,
                        saved_id: &mut saved_id,
                    },
                    bronze_capture::NoFeedback,
                );
                let mut ingress = CaptureIngressContext::empty();
                ingress.route = INGRESS_ROUTE_MENU;
                coordinator.submit(ingress);
                coordinator.drain();
                let terminal = coordinator
                    .receipt(1)
                    .map(|receipt| receipt.terminal)
                    .unwrap_or(Terminal::Failed);
                drop(coordinator);
                if terminal == Terminal::Saved {
                    let prior = FocusSnapshot {
                        owner: FocusOwner::Source,
                        source_token: 0,
                    };
                    let owner = apply_capture_success(
                        CaptureMode::CaptureOnly,
                        prior,
                        announcer,
                        webview_visible,
                    );
                    debug_assert_eq!(owner, FocusOwner::Source);
                    let _ = CAPTURE_ONLY_REVEALS_PANEL;
                }
                Ok(CapturePersistOutcome {
                    terminal,
                    reason: if terminal == Terminal::Saved {
                        "ok"
                    } else {
                        "failed"
                    },
                    item_id: saved_id,
                })
            }
            (AxOutcome::ProtectedContent | AxOutcome::ProtectionUnknown, _) => {
                Ok(CapturePersistOutcome {
                    terminal: Terminal::Rejected,
                    reason: "protected",
                    item_id: None,
                })
            }
            (AxOutcome::NoSelection | AxOutcome::FocusedElementMissing, _) => {
                Ok(CapturePersistOutcome {
                    terminal: Terminal::Rejected,
                    reason: "no_selection",
                    item_id: None,
                })
            }
            (AxOutcome::AppExcluded, _) => Ok(CapturePersistOutcome {
                terminal: Terminal::Rejected,
                reason: "app_excluded",
                item_id: None,
            }),
            (AxOutcome::AccessibilityDenied, _) => Ok(CapturePersistOutcome {
                terminal: Terminal::Rejected,
                reason: "accessibility",
                item_id: None,
            }),
            (AxOutcome::SelectionTooLarge | AxOutcome::InvalidTextEncoding, _) => {
                Ok(CapturePersistOutcome {
                    terminal: Terminal::Failed,
                    reason: "failed",
                    item_id: None,
                })
            }
            (AxOutcome::Captured { .. }, None) => Ok(CapturePersistOutcome {
                terminal: Terminal::Failed,
                reason: "failed",
                item_id: None,
            }),
        }
    }

    pub fn add_composer(&mut self, body: String) -> Result<QueueItemDto, String> {
        if body.is_empty() {
            return Err("composer_empty".into());
        }
        let now = now_ms();
        let section = self
            .store
            .ensure_inbox(now)
            .map_err(|_| "inbox_seed_failed")?;
        let id = next_item_id(now);
        self.store
            .add_from_composer(
                &ComposerDraft {
                    body,
                    content_language: None,
                },
                ComposerChord::CmdEnter,
                false,
                &section,
                &id,
                now,
            )
            .map_err(|_| "composer_store_failed")?;
        self.store
            .get_item(&id)
            .map(QueueItemDto::from)
            .map_err(|_| "composer_store_failed".into())
    }

    pub fn add_captured(
        &mut self,
        body: String,
        source_app_name: Option<String>,
        source_bundle_id: Option<String>,
    ) -> Result<QueueItemDto, String> {
        if body.is_empty() {
            return Err("composer_empty".into());
        }
        let now = now_ms();
        let section = self
            .store
            .ensure_inbox(now)
            .map_err(|_| "inbox_seed_failed")?;
        let id = next_item_id(now);
        self.store
            .add_from_capture(
                &ComposerDraft {
                    body,
                    content_language: None,
                },
                source_app_name.as_deref(),
                source_bundle_id.as_deref(),
                &section,
                &id,
                now,
            )
            .map_err(|_| "capture_store_failed")?;
        self.store
            .get_item(&id)
            .map(QueueItemDto::from)
            .map_err(|_| "capture_store_failed".into())
    }

    pub fn apply_action(&mut self, id: &str, action: &str) -> Result<Vec<QueueItemDto>, String> {
        let now = now_ms();
        let parsed = parse_action(action)?;
        self.store
            .apply_queue_action(id, parsed, now)
            .map_err(|err| format!("{err:?}"))?;
        self.list_queue(false)
    }

    pub fn edit_item(&mut self, id: &str, body: &str) -> Result<QueueItemDto, String> {
        self.store
            .edit_item_body(id, body, now_ms())
            .map_err(|err| format!("{err:?}"))?;
        self.store
            .get_item(id)
            .map(QueueItemDto::from)
            .map_err(|_| "not_found".into())
    }

    pub fn copy_items(
        &mut self,
        item_ids: &[String],
        profile: &str,
        board: &mut impl Pasteboard,
    ) -> Result<String, String> {
        let started = now_ms();
        let result = self.copy_items_apply(item_ids, profile, board);
        let (result_code, store_result) = match &result {
            Ok(_) => ("ok", "ok"),
            Err(err) if err == "copy_empty" => ("no_selection", "failed"),
            Err(_) => ("persist_failed", "failed"),
        };
        self.record_terminal_diag(TerminalDiag {
            stage: "clipboard",
            result: result_code,
            trigger: "manual_clipboard",
            provider: "clipboard",
            source_bundle_id: None,
            store_result: Some(store_result),
            duration_ms: Some(now_ms().saturating_sub(started)),
        });
        result
    }

    fn copy_items_apply(
        &mut self,
        item_ids: &[String],
        profile: &str,
        board: &mut impl Pasteboard,
    ) -> Result<String, String> {
        let rows = self
            .store
            .list_items(false)
            .map_err(|_| "queue_list_failed")?;
        let selected: Vec<&str> = if item_ids.is_empty() {
            rows.iter()
                .filter(|row| {
                    row.status == "queued" || row.status == "copied" || row.status == "active"
                })
                .map(|row| row.body.as_str())
                .collect()
        } else {
            rows.iter()
                .filter(|row| item_ids.iter().any(|id| id == &row.id))
                .map(|row| row.body.as_str())
                .collect()
        };
        if selected.is_empty() {
            return Err("copy_empty".into());
        }
        let named = named_output_profile(profile);
        let mut life = bronze_domain::Lifecycle::Queued;
        let text = copy_items(&selected, &named, board, &mut life).map_err(copy_err)?;
        if life == bronze_domain::Lifecycle::Copied {
            let now = now_ms();
            let targets: Vec<String> = if item_ids.is_empty() {
                rows.iter()
                    .filter(|row| row.status == "queued")
                    .map(|row| row.id.clone())
                    .collect()
            } else {
                item_ids.to_vec()
            };
            for id in targets {
                let _ = self.store.mark_copied(&id, now);
            }
        }
        Ok(text)
    }

    pub fn search_library(&self, query: &str) -> Result<(Vec<QueueItemDto>, usize), String> {
        let hits = self
            .store
            .search_placeholder(query)
            .map_err(|_| "search_failed")?;
        let mut items = Vec::new();
        for hit in &hits {
            if let Ok(row) = self.store.get_item(&hit.item_id) {
                items.push(QueueItemDto::from(row));
            }
        }
        Ok((items, hits.len()))
    }

    pub fn settings(&self) -> SettingsV1 {
        self.settings.clone()
    }

    pub fn replace_settings(&mut self, next: SettingsV1) -> Result<SettingsV1, String> {
        next.validate().map_err(|_| "settings_invalid")?;
        let mut next = next;
        next.sync_motion_fields();
        let _ = resolve_title_model(&mut next);
        let chord = next.capture.standard_chord.clone();
        self.settings = next;
        persist_settings_file(&self.data_dir, &self.settings)?;
        apply_title_engine(&self.settings);
        apply_login_item(&self.settings);
        self.shortcuts
            .register(chord, &mut SessionRegistrar, keep_menu_manual())
            .map_err(register_error_code)?;
        self.sync_standard_chord()?;
        Ok(self.settings.clone())
    }

    pub fn reset_field(&mut self, field_id: &str) -> Result<SettingsV1, String> {
        self.settings
            .reset_field(field_id)
            .map_err(|_| "settings_unknown_field")?;
        if field_id == "capture.standardChord" {
            self.shortcuts
                .restore_default(
                    ShortcutActionId::CaptureSelection,
                    &mut SessionRegistrar,
                    keep_menu_manual(),
                )
                .map_err(register_error_code)?;
        }
        if field_id == "general.titleModel" {
            let _ = resolve_title_model(&mut self.settings);
        }
        apply_title_engine(&self.settings);
        apply_login_item(&self.settings);
        self.sync_standard_chord()?;
        Ok(self.settings.clone())
    }

    pub fn reset_group(&mut self, group: &str) -> Result<SettingsV1, String> {
        let parsed = parse_group(group)?;
        self.settings.reset_group(parsed);
        if parsed == SettingsGroup::Capture {
            self.shortcuts
                .restore_default(
                    ShortcutActionId::CaptureSelection,
                    &mut SessionRegistrar,
                    keep_menu_manual(),
                )
                .map_err(register_error_code)?;
        }
        if parsed == SettingsGroup::General {
            let _ = resolve_title_model(&mut self.settings);
        }
        apply_title_engine(&self.settings);
        apply_login_item(&self.settings);
        self.sync_standard_chord()?;
        Ok(self.settings.clone())
    }

    pub fn reset_all(&mut self) -> Result<SettingsV1, String> {
        self.settings.reset_all_preserving_content();
        let _ = resolve_title_model(&mut self.settings);
        apply_title_engine(&self.settings);
        apply_login_item(&self.settings);
        self.shortcuts = ShortcutRegistry::seeded();
        self.sync_standard_chord()?;
        Ok(self.settings.clone())
    }

    pub fn list_shortcuts(&self) -> Vec<ShortcutRowDto> {
        ShortcutActionId::ALL
            .iter()
            .copied()
            .map(|action| shortcut_row(self.shortcuts.get(action)))
            .collect()
    }

    pub fn record_shortcut(
        &mut self,
        input: ShortcutRecordInput,
    ) -> Result<Vec<ShortcutRowDto>, String> {
        let action = ShortcutActionId::parse(&input.action).map_err(|_| "unknown_shortcut")?;
        let mut candidate = binding_from_record(&input, self.shortcuts.get(action).revision + 1)?;
        if input.skip_test {
            skip_test_marks_untested(&mut candidate);
        }
        self.shortcuts
            .register(candidate, &mut SessionRegistrar, keep_menu_manual())
            .map_err(register_error_code)?;
        self.sync_standard_chord()?;
        Ok(self.list_shortcuts())
    }

    pub fn restore_shortcut(&mut self, action: &str) -> Result<Vec<ShortcutRowDto>, String> {
        let action = ShortcutActionId::parse(action).map_err(|_| "unknown_shortcut")?;
        self.shortcuts
            .restore_default(action, &mut SessionRegistrar, keep_menu_manual())
            .map_err(register_error_code)?;
        self.sync_standard_chord()?;
        Ok(self.list_shortcuts())
    }

    fn sync_standard_chord(&mut self) -> Result<(), String> {
        self.settings.capture.standard_chord = self.shortcuts.standard_chord().clone();
        persist_settings_file(&self.data_dir, &self.settings)?;
        self.store
            .persist_shortcuts(&self.shortcuts)
            .map_err(|_| "shortcuts_persist_failed".to_string())?;
        self.apply_live_capture_gesture();
        Ok(())
    }

    pub fn backup_now(&self) -> Result<String, String> {
        let dest = rust_owned_file(&self.data_dir, "backups", "bronze.sqlite")?;
        let owned = dest.to_string_lossy().into_owned();
        let accepted = accept_native_path(PathSource::RustPicker, &owned)
            .map_err(|_| "webview_path_rejected")?;
        self.store
            .online_backup(accepted)
            .map_err(|_| "backup_failed")?;
        Ok(accepted.display().to_string())
    }

    pub fn export_library(&self) -> Result<String, String> {
        let dest = rust_owned_dir(&self.data_dir, "exports", "bronze-export")?;
        let owned = dest.to_string_lossy().into_owned();
        let accepted = accept_native_path(PathSource::RustPicker, &owned)
            .map_err(|_| "webview_path_rejected")?;
        self.store
            .export_archive(accepted, now_ms(), Overwrite::Replace)
            .map_err(|_| "export_failed")?;
        Ok(accepted.display().to_string())
    }

    pub fn import_library(&mut self, snapshot: &str) -> Result<usize, String> {
        if snapshot.contains('/') || snapshot.contains('\\') || snapshot.contains("..") {
            return Err("webview_path_rejected".into());
        }
        let dest = self.data_dir.join("exports").join(snapshot);
        let owned = dest.to_string_lossy().into_owned();
        let accepted = accept_native_path(PathSource::RustPicker, &owned)
            .map_err(|_| "webview_path_rejected")?;
        if !accepted.exists() {
            return Err("import_missing".into());
        }
        let preview = self
            .store
            .import_commit(accepted, ImportStrategy::Replace)
            .map_err(|_| "import_failed")?;
        Ok(preview.item_count)
    }

    pub fn latest_export_snapshot(&self) -> Result<Option<String>, String> {
        let root = self.data_dir.join("exports");
        if !root.exists() {
            return Ok(None);
        }
        let mut names: Vec<String> = fs::read_dir(&root)
            .map_err(|err| err.to_string())?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();
        names.sort();
        Ok(names.pop())
    }

    pub fn preview_settings_export(&self) -> Result<SettingsExportPreviewDto, String> {
        Ok(SettingsExportPreviewDto::from_preview(
            self.settings_export_preview()?,
        ))
    }

    pub fn export_settings_to_path(&self, dest: &Path) -> Result<String, String> {
        let accepted = accept_settings_file_path(dest.to_str().unwrap_or_default())
            .map_err(|_| "settings_path_invalid")?;
        accept_native_path(
            PathSource::RustPicker,
            accepted.to_str().ok_or("settings_path_invalid")?,
        )
        .map_err(|_| "webview_path_rejected")?;
        if let Some(parent) = accepted.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        let preview = self.settings_export_preview()?;
        let raw = serde_json::to_string_pretty(&preview.payload).map_err(|_| "settings_invalid")?;
        fs::write(&accepted, raw).map_err(|err| err.to_string())?;
        Ok(accepted
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("bronze-settings.json")
            .to_string())
    }

    pub fn import_settings_from_path(&mut self, path: &Path) -> Result<SettingsV1, String> {
        let accepted = accept_settings_file_path(path.to_str().unwrap_or_default())
            .map_err(|_| "settings_path_invalid")?;
        accept_native_path(
            PathSource::RustPicker,
            accepted.to_str().ok_or("settings_path_invalid")?,
        )
        .map_err(|_| "webview_path_rejected")?;
        let bytes = read_settings_import_bytes(&accepted).map_err(settings_file_error_code)?;
        let raw = String::from_utf8(bytes).map_err(|_| "settings_invalid")?;
        let document = parse_settings_import(&raw).map_err(settings_import_error_code)?;
        self.replace_settings(document.settings)?;
        if let Some(shortcuts) = document.shortcuts {
            for binding in shortcuts {
                let _ = self
                    .shortcuts
                    .register(binding, &mut SessionRegistrar, keep_menu_manual());
            }
            self.sync_standard_chord()?;
        }
        if let Some(profiles) = document.profiles {
            self.store
                .replace_output_profiles(&profiles)
                .map_err(|_| "settings_profiles_invalid")?;
        }
        debug_assert_eq!(ADR_018_STATUS, "Proposed");
        Ok(self.settings.clone())
    }

    pub fn preview_support_text(&self) -> Result<String, String> {
        let report = self.build_support_report();
        render_support_report(&report).map_err(|_| "support_secret".into())
    }

    pub fn export_support_to_path(&self, dest: &Path) -> Result<String, String> {
        let accepted = accept_support_file_path(dest.to_str().unwrap_or_default())
            .map_err(|_| "support_path_invalid")?;
        accept_native_path(
            PathSource::RustPicker,
            accepted.to_str().ok_or("support_path_invalid")?,
        )
        .map_err(|_| "webview_path_rejected")?;
        if let Some(parent) = accepted.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        let body = self.preview_support_text()?;
        fs::write(&accepted, body).map_err(|err| err.to_string())?;
        Ok(accepted
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("bronze-support.txt")
            .to_string())
    }

    fn build_support_report(&self) -> SupportReport {
        let now = now_ms();
        let since = now.saturating_sub(LAST_HOUR_MS as i64);
        let events = self
            .store
            .list_diagnostics_since(since)
            .unwrap_or_default()
            .into_iter()
            .map(|row| SupportEventLine {
                occurred_at_ms: row.occurred_at_ms as u64,
                request_id: row.request_id,
                stage: row.stage,
                result: row.result_code,
                duration_ms: row.duration_ms.and_then(|ms| u32::try_from(ms).ok()),
                trigger_kind: row.trigger_kind,
                source_bundle_id: row.source_bundle_id,
            })
            .collect();
        let recent_sources = self
            .store
            .list_sources_since(since)
            .unwrap_or_default()
            .into_iter()
            .map(|row| SupportSourceApp {
                bundle_id: row.bundle_id.unwrap_or_default(),
                app_name: row.app_name.unwrap_or_default(),
            })
            .collect();
        let counts = self.store.queue_status_counts().unwrap_or_default();
        let engine = bronze_title_model::current_status();
        SupportReport {
            generated_at_ms: now as u64,
            app_version: env!("CARGO_PKG_VERSION").into(),
            toolchain: "1.98.1".into(),
            rust_min: env!("CARGO_PKG_RUST_VERSION").into(),
            tauri_version: "2.11.3".into(),
            rusqlite_version: "0.32".into(),
            settings_schema: SCHEMA_VERSION.to_string(),
            store_schema: self
                .store
                .schema_version()
                .unwrap_or(APP_SCHEMA_VERSION)
                .to_string(),
            os: std::env::consts::OS.into(),
            os_version: macos_os_version(),
            arch: std::env::consts::ARCH.into(),
            model: macos_model(),
            data_path: redact_home_path(&self.data_dir.to_string_lossy()),
            login_item_setting: if self.settings.general.launch_at_login {
                "on".into()
            } else {
                "off".into()
            },
            login_item_status: bronze_platform_macos::login_item_status().as_str().into(),
            title_engine_tier: engine.tier.as_str().into(),
            title_engine_phase: engine.phase.as_str().into(),
            permissions: support_permission_rows(),
            queue: SupportQueueCounts {
                queued: counts.queued,
                copied: counts.copied,
                active: counts.active,
                done: counts.done,
                skipped: counts.skipped,
                trashed: counts.trashed,
            },
            excluded_bundle_ids: self.settings.privacy.excluded_bundle_ids.clone(),
            recent_sources,
            events,
            last_hour_ms: LAST_HOUR_MS,
        }
    }

    fn record_terminal_diag(&self, diag: TerminalDiag) {
        let now = now_ms();
        let row = DiagnosticEventRow {
            id: next_diag_id(now),
            occurred_at_ms: now,
            request_id: format!("r{now}"),
            stage: diag.stage.into(),
            result_code: diag.result.into(),
            duration_ms: diag.duration_ms,
            trigger_kind: diag.trigger.into(),
            provider_kind: Some(diag.provider.into()),
            permission_state: None,
            source_bundle_id: diag.source_bundle_id,
            app_schema_version: APP_SCHEMA_VERSION,
            queue_depth: None,
            overflow_count: None,
            tap_health: None,
            store_result_code: diag.store_result.map(str::to_string),
            build_id: env!("CARGO_PKG_VERSION").into(),
        };
        let _ = self.store.insert_diagnostic_event(&row);
    }

    fn settings_export_preview(&self) -> Result<SettingsExportPreview, String> {
        let profiles = self
            .store
            .list_output_profiles()
            .map_err(|_| "settings_profiles_invalid")?;
        export_settings_document(
            &self.settings,
            &self.shortcuts.all_bindings(),
            &profiles,
            &std::collections::BTreeMap::new(),
            &rfc3339_utc(now_ms()),
        )
        .map_err(|_| "settings_invalid".to_string())
    }
}

fn parse_action(action: &str) -> Result<QueueAction, String> {
    match action {
        "complete" => Ok(QueueAction::Complete),
        "skip" => Ok(QueueAction::Skip),
        "trash" => Ok(QueueAction::Trash),
        "moveUp" => Ok(QueueAction::MoveUp),
        "moveDown" => Ok(QueueAction::MoveDown),
        _ => Err("unknown_action".into()),
    }
}

fn shortcut_row(binding: &ShortcutBinding) -> ShortcutRowDto {
    let trigger = match binding.trigger {
        TriggerKind::Accelerator => "accelerator",
        TriggerKind::ModifierDoubleTap => "modifier_double_tap",
        TriggerKind::Disabled => "disabled",
    };
    let tested = binding.tested.map(|state| match state {
        TestedState::Tested => "tested".into(),
        TestedState::Untested => "untested".into(),
        TestedState::Skipped => "skipped".into(),
    });
    let scope = match shortcut_scope(binding.action) {
        ShortcutScope::Global => "global",
        ShortcutScope::AppLocal => "appLocal",
    };
    ShortcutRowDto {
        action: binding.action.as_str().into(),
        trigger: trigger.into(),
        modifiers: binding.modifiers.iter().map(modifier_name).collect(),
        logical_key: binding.logical_key.clone(),
        tap_count: effective_tap_count(binding),
        enabled: binding.enabled,
        is_default: is_default_binding(binding),
        scope: scope.into(),
        tested,
    }
}

fn modifier_name(modifier: &Modifier) -> String {
    match modifier {
        Modifier::Command => "Command".into(),
        Modifier::Option => "Option".into(),
        Modifier::Control => "Control".into(),
        Modifier::Shift => "Shift".into(),
        Modifier::Fn => "Fn".into(),
    }
}

fn parse_trigger(raw: &str) -> Result<TriggerKind, String> {
    match raw {
        "accelerator" => Ok(TriggerKind::Accelerator),
        "modifier_double_tap" => Ok(TriggerKind::ModifierDoubleTap),
        "disabled" => Ok(TriggerKind::Disabled),
        _ => Err("invalid_shortcut_trigger".into()),
    }
}

fn parse_modifier(raw: &str) -> Result<Modifier, String> {
    match raw {
        "Command" => Ok(Modifier::Command),
        "Option" => Ok(Modifier::Option),
        "Control" => Ok(Modifier::Control),
        "Shift" => Ok(Modifier::Shift),
        "Fn" => Ok(Modifier::Fn),
        _ => Err("invalid_shortcut_modifier".into()),
    }
}

fn binding_from_record(
    input: &ShortcutRecordInput,
    revision: u32,
) -> Result<ShortcutBinding, String> {
    let action = ShortcutActionId::parse(&input.action).map_err(|_| "unknown_shortcut")?;
    let trigger = parse_trigger(&input.trigger)?;
    let modifiers = input
        .modifiers
        .iter()
        .map(|raw| parse_modifier(raw))
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(key) = input.logical_key.as_deref() {
        if !logical_key_allowed(key) {
            return Err("invalid_shortcut_key".into());
        }
    }
    if let Some(count) = input.tap_count {
        if !(2..=MAX_MODIFIER_TAPS).contains(&count) {
            return Err("invalid_shortcut_tap_count".into());
        }
    }
    let mut binding = default_shortcut_binding(action);
    binding.trigger = trigger;
    binding.modifiers = modifiers;
    binding.logical_key = input.logical_key.clone();
    binding.tap_count = input.tap_count;
    binding.enabled = trigger != TriggerKind::Disabled;
    binding.revision = revision;
    apply_recorded_double_tap_timing(&mut binding);
    Ok(binding)
}

fn parse_group(group: &str) -> Result<SettingsGroup, String> {
    match group {
        "general" => Ok(SettingsGroup::General),
        "capture" => Ok(SettingsGroup::Capture),
        "panel" => Ok(SettingsGroup::Panel),
        "copy" => Ok(SettingsGroup::Copy),
        "privacy" => Ok(SettingsGroup::Privacy),
        "data" => Ok(SettingsGroup::Data),
        "accessibility" => Ok(SettingsGroup::Accessibility),
        _ => Err("unknown_group".into()),
    }
}

fn physical_ram_bytes() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("/usr/sbin/sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        String::from_utf8(output.stdout).ok()?.trim().parse().ok()
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

fn title_model_from_tier(tier: TitleTier) -> TitleModelId {
    match tier {
        TitleTier::Extractive => TitleModelId::Extractive,
        TitleTier::Smol135 => TitleModelId::Smol135,
        TitleTier::Smol360 => TitleModelId::Smol360,
        TitleTier::Qwen05 => TitleModelId::Qwen05,
    }
}

fn title_tier_from_model(id: TitleModelId) -> TitleTier {
    match id {
        TitleModelId::Unset | TitleModelId::Extractive => TitleTier::Extractive,
        TitleModelId::Smol135 => TitleTier::Smol135,
        TitleModelId::Smol360 => TitleTier::Smol360,
        TitleModelId::Qwen05 => TitleTier::Qwen05,
    }
}

fn resolve_title_model(settings: &mut SettingsV1) -> bool {
    if !settings.general.title_model.is_unset() {
        return false;
    }
    settings.general.title_model = title_model_from_tier(auto_pick_title_tier(
        physical_ram_bytes(),
        &present_gguf_tiers(),
    ));
    true
}

fn apply_title_engine(settings: &SettingsV1) {
    request_tier(title_tier_from_model(settings.general.title_model));
}

fn apply_login_item(settings: &SettingsV1) {
    let _ = bronze_platform_macos::apply_login_item(settings.general.launch_at_login);
}

fn load_settings_file(data_dir: &Path) -> Result<SettingsV1, String> {
    let path = data_dir.join("settings.json");
    if !path.exists() {
        return Ok(SettingsV1::defaults());
    }
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    SettingsV1::from_json(&raw).map_err(|_| "settings_invalid".into())
}

fn persist_settings_file(data_dir: &Path, settings: &SettingsV1) -> Result<(), String> {
    let raw = settings.to_json().map_err(|_| "settings_invalid")?;
    fs::write(data_dir.join("settings.json"), raw).map_err(|err| err.to_string())
}

fn rust_owned_file(data_dir: &Path, folder: &str, file: &str) -> Result<PathBuf, String> {
    let dir = data_dir.join(folder);
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    Ok(dir.join(format!("{}-{}", now_ms(), file)))
}

fn rust_owned_dir(data_dir: &Path, folder: &str, prefix: &str) -> Result<PathBuf, String> {
    let dir = data_dir
        .join(folder)
        .join(format!("{}-{}", prefix, now_ms()));
    Ok(dir)
}

fn next_item_id(now: i64) -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    format!(
        "i{now}-{}",
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

fn next_diag_id(now: i64) -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    format!(
        "d{now}-{}",
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

fn capture_stage(reason: &str) -> &'static str {
    match reason {
        "ok" | "failed" => "store",
        "no_selection" => "selection",
        _ => "ax",
    }
}

fn capture_result_code(reason: &str) -> &'static str {
    match reason {
        "ok" => "ok",
        "app_excluded" => "app_excluded",
        "protected" => "protected_content",
        "no_selection" => "no_selection",
        "accessibility" => "context_unavailable",
        _ => "persist_failed",
    }
}

fn support_permission_rows() -> Vec<SupportNamedState> {
    #[cfg(test)]
    {
        vec![
            SupportNamedState {
                name: "input_monitoring".into(),
                state: "unavailable".into(),
            },
            SupportNamedState {
                name: "accessibility".into(),
                state: "unavailable".into(),
            },
            SupportNamedState {
                name: "capture_pipeline_self_test".into(),
                state: "unknown".into(),
            },
        ]
    }
    #[cfg(not(test))]
    {
        let snapshot = snapshot_from_preflight(&MacosPreflightHost);
        vec![
            SupportNamedState {
                name: "input_monitoring".into(),
                state: snapshot.input_monitoring.as_str().into(),
            },
            SupportNamedState {
                name: "accessibility".into(),
                state: snapshot.accessibility.as_str().into(),
            },
            SupportNamedState {
                name: "capture_pipeline_self_test".into(),
                state: snapshot.capture_pipeline_self_test.as_str().into(),
            },
        ]
    }
}

fn macos_os_version() -> String {
    let Ok(raw) = fs::read_to_string("/System/Library/CoreServices/SystemVersion.plist") else {
        return "unavailable".into();
    };
    match plist_string_value(&raw, "ProductVersion") {
        Some(version) if !version.is_empty() => version,
        _ => "unavailable".into(),
    }
}

fn plist_string_value(plist: &str, key: &str) -> Option<String> {
    let needle = format!("<key>{key}</key>");
    let after = plist.split(&needle).nth(1)?;
    let start = after.find("<string>")? + 8;
    let end = after.get(start..)?.find("</string>")?;
    Some(after[start..start + end].trim().to_string())
}

fn macos_model() -> String {
    let Ok(output) = std::process::Command::new("/usr/sbin/sysctl")
        .args(["-n", "hw.model"])
        .output()
    else {
        return "unavailable".into();
    };
    if !output.status.success() {
        return "unavailable".into();
    }
    let model = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if model.is_empty() || bronze_diagnostics::contains_forbidden_payload(&model) {
        "unavailable".into()
    } else {
        model
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn rfc3339_utc(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let days = secs.div_euclid(86_400);
    let tod = secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    let hour = tod / 3600;
    let min = (tod % 3600) / 60;
    let sec = tod % 60;
    format!("{y:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}Z")
}

fn settings_file_error_code(err: SettingsFileError) -> String {
    match err {
        SettingsFileError::TooLarge => "settings_import_too_large".into(),
        SettingsFileError::NotJson => "settings_import_not_json".into(),
        SettingsFileError::Invalid => "settings_path_invalid".into(),
    }
}

fn settings_import_error_code(err: SettingsImportError) -> String {
    match err {
        SettingsImportError::UnknownVersion => "settings_unknown_version".into(),
        SettingsImportError::WrongFormat => "settings_wrong_format".into(),
        SettingsImportError::TooLarge => "settings_import_too_large".into(),
        SettingsImportError::ForbiddenContent => "settings_forbidden".into(),
        SettingsImportError::InvalidSchema | SettingsImportError::InvalidJson => {
            "settings_invalid".into()
        }
    }
}

fn copy_err(err: CopyError) -> String {
    match err {
        CopyError::Pasteboard => "pasteboard".into(),
        CopyError::SyntheticPasteForbidden => "synthetic_paste_forbidden".into(),
    }
}

pub struct MacPasteboard;

impl Pasteboard for MacPasteboard {
    fn write_text(&mut self, text: &str) -> Result<(), CopyError> {
        self.write_plain_and_html(text, "")
    }

    fn write_plain_and_html(&mut self, plain: &str, html: &str) -> Result<(), CopyError> {
        bronze_platform_macos::native_pasteboard_write(plain, html)
            .map_err(|_| CopyError::Pasteboard)
    }
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn list_queue_items(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    include_trashed: Option<bool>,
) -> Result<Vec<QueueItemDto>, String> {
    lock_session(&session)?.list_queue(include_trashed.unwrap_or(false))
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn list_overview_items(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
) -> Result<Vec<QueueItemDto>, String> {
    lock_session(&session)?.list_overview()
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn add_composer_item(
    app: tauri::AppHandle,
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    body: String,
) -> Result<QueueItemDto, String> {
    let dto = lock_session(&session)?.add_composer(body.clone())?;
    crate::title_refine::schedule(&app, dto.id.clone(), body);
    Ok(dto)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn apply_queue_item_action(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    id: String,
    action: String,
) -> Result<Vec<QueueItemDto>, String> {
    lock_session(&session)?.apply_action(&id, &action)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn edit_queue_item(
    app: tauri::AppHandle,
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    id: String,
    body: String,
) -> Result<QueueItemDto, String> {
    let dto = lock_session(&session)?.edit_item(&id, &body)?;
    crate::title_refine::schedule(&app, id, body);
    Ok(dto)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn copy_queue_items(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    item_ids: Option<Vec<String>>,
    profile: Option<String>,
) -> Result<String, String> {
    let ids = item_ids.unwrap_or_default();
    let profile = profile.unwrap_or_else(|| "plain".into());
    lock_session(&session)?.copy_items(&ids, &profile, &mut MacPasteboard)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn search_library_items(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    query: String,
) -> Result<Vec<QueueItemDto>, String> {
    Ok(lock_session(&session)?.search_library(&query)?.0)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn load_settings_v1(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
) -> Result<SettingsV1, String> {
    Ok(lock_session(&session)?.settings())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn login_item_status() -> String {
    bronze_platform_macos::login_item_status()
        .as_str()
        .to_string()
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn save_settings_v1(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    settings: SettingsV1,
) -> Result<SettingsV1, String> {
    lock_session(&session)?.replace_settings(settings)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn reset_settings_field(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    field_id: String,
) -> Result<SettingsV1, String> {
    lock_session(&session)?.reset_field(&field_id)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn reset_settings_group(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    group: String,
) -> Result<SettingsV1, String> {
    lock_session(&session)?.reset_group(&group)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn reset_settings_all(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
) -> Result<SettingsV1, String> {
    lock_session(&session)?.reset_all()
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleModelTierDto {
    pub id: String,
    pub present: bool,
    pub filename: String,
    pub bytes: u64,
    pub vendor_command: String,
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn list_title_models() -> Vec<TitleModelTierDto> {
    let present = present_gguf_tiers();
    let mut rows = vec![TitleModelTierDto {
        id: TitleTier::Extractive.as_str().into(),
        present: true,
        filename: String::new(),
        bytes: 0,
        vendor_command: String::new(),
    }];
    for spec in GGUF_TIERS {
        rows.push(TitleModelTierDto {
            id: spec.tier.as_str().into(),
            present: present.contains(&spec.tier),
            filename: spec.filename.into(),
            bytes: spec.bytes,
            vendor_command: spec.tier.vendor_command().into(),
        });
    }
    rows
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn search_settings_fields(query: String) -> Vec<String> {
    search_settings(&query)
        .into_iter()
        .map(|field| field.id.to_string())
        .collect()
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn list_shortcuts(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
) -> Result<Vec<ShortcutRowDto>, String> {
    Ok(lock_session(&session)?.list_shortcuts())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn record_shortcut(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    input: ShortcutRecordInput,
) -> Result<Vec<ShortcutRowDto>, String> {
    lock_session(&session)?.record_shortcut(input)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn restore_shortcut(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    action: String,
) -> Result<Vec<ShortcutRowDto>, String> {
    lock_session(&session)?.restore_shortcut(&action)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn list_installed_apps() -> Result<Vec<InstalledAppDto>, String> {
    bronze_platform_macos::try_list_installed_apps()
        .map(|apps| {
            apps.into_iter()
                .filter(|app| bronze_platform_macos::is_safe_bundle_id(&app.bundle_id))
                .map(InstalledAppDto::from)
                .collect()
        })
        .ok_or_else(|| "apps_unavailable".into())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn pick_installed_app() -> Result<InstalledAppDto, String> {
    match bronze_platform_macos::try_pick_installed_app() {
        bronze_platform_macos::PickInstalledApp::Picked(app) => {
            if !bronze_platform_macos::is_safe_bundle_id(&app.bundle_id) {
                return Err("invalid_app".into());
            }
            Ok(InstalledAppDto::from(app))
        }
        bronze_platform_macos::PickInstalledApp::Cancelled => Err("picker_cancelled".into()),
        bronze_platform_macos::PickInstalledApp::Unavailable => Err("picker_unavailable".into()),
        bronze_platform_macos::PickInstalledApp::Invalid => Err("invalid_app".into()),
    }
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn app_icon_data_url(bundle_id: String) -> Result<Option<String>, String> {
    if !bronze_platform_macos::is_safe_bundle_id(&bundle_id) {
        return Err("invalid_bundle_id".into());
    }
    Ok(source_app_icon_data_url(Some(&bundle_id), None))
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn ui_locale(session: tauri::State<std::sync::Mutex<LiveSession>>) -> String {
    lock_session(&session)
        .map(|session| session.ui_locale().to_string())
        .unwrap_or_else(|_| HAND_TEST_UI_LOCALE.into())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn ui_catalog(session: tauri::State<std::sync::Mutex<LiveSession>>) -> UiCatalogDto {
    lock_session(&session)
        .map(|session| session.ui_catalog())
        .unwrap_or_else(|_| LiveSession::unavailable_catalog())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn backup_library_now(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    requested_path: Option<String>,
) -> Result<String, String> {
    reject_webview_path(requested_path)?;
    lock_session(&session)?.backup_now()
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn export_library_archive(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    requested_path: Option<String>,
) -> Result<String, String> {
    reject_webview_path(requested_path)?;
    lock_session(&session)?.export_library()
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn import_library_archive(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    snapshot: Option<String>,
    requested_path: Option<String>,
) -> Result<usize, String> {
    reject_webview_path(requested_path)?;
    let mut live = lock_session(&session)?;
    let name = match snapshot {
        Some(name) => name,
        None => live
            .latest_export_snapshot()?
            .ok_or_else(|| "import_missing".to_string())?,
    };
    live.import_library(&name)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn preview_settings_export(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
) -> Result<SettingsExportPreviewDto, String> {
    lock_session(&session)?.preview_settings_export()
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn export_settings_file(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    requested_path: Option<String>,
) -> Result<String, String> {
    reject_webview_path(requested_path)?;
    let dest = match try_pick_settings_export_path() {
        PickSettingsFile::Picked(path) => path,
        PickSettingsFile::Cancelled => return Err("picker_cancelled".into()),
        PickSettingsFile::Unavailable => return Err("picker_unavailable".into()),
    };
    lock_session(&session)?.export_settings_to_path(&dest)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn import_settings_file(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    requested_path: Option<String>,
) -> Result<SettingsV1, String> {
    reject_webview_path(requested_path)?;
    let src = match try_pick_settings_import_path() {
        PickSettingsFile::Picked(path) => path,
        PickSettingsFile::Cancelled => return Err("picker_cancelled".into()),
        PickSettingsFile::Unavailable => return Err("picker_unavailable".into()),
    };
    lock_session(&session)?.import_settings_from_path(&src)
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn preview_support_bundle(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
) -> Result<String, String> {
    lock_session(&session)?.preview_support_text()
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn export_support_file(
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    requested_path: Option<String>,
) -> Result<String, String> {
    reject_webview_path(requested_path)?;
    let dest = match try_pick_support_export_path() {
        PickSettingsFile::Picked(path) => path,
        PickSettingsFile::Cancelled => return Err("picker_cancelled".into()),
        PickSettingsFile::Unavailable => return Err("picker_unavailable".into()),
    };
    lock_session(&session)?.export_support_to_path(&dest)
}

fn reject_webview_path(requested_path: Option<String>) -> Result<(), String> {
    if requested_path
        .as_deref()
        .is_some_and(|path| !path.is_empty())
    {
        return Err("webview_path_rejected".into());
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn lock_session(
    session: &std::sync::Mutex<LiveSession>,
) -> Result<std::sync::MutexGuard<'_, LiveSession>, String> {
    Ok(session
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()))
}

#[cfg(test)]
mod live_session_tests {
    use super::*;
    use crate::copy::FakePasteboard;
    use bronze_storage::QUE_007_COMPLETE;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(1);

    fn open_session() -> LiveSession {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("bronze-live-{n}-{}", now_ms()));
        LiveSession::open(dir).expect("session")
    }

    #[test]
    fn composer_persists_and_queue_actions_mutate() {
        let mut session = open_session();
        assert!(AX_CAPTURE_LIVE);
        assert!(!CAPTURE_ONLY_REVEALS_PANEL);
        assert_eq!(session.ui_locale(), "en");
        let added = session.add_composer("  park me  ".into()).expect("add");
        assert_eq!(added.body, "  park me  ");
        assert_eq!(
            added.title,
            Some(bronze_domain::compact_title("  park me  "))
        );
        assert_eq!(added.content_language, "und");
        assert_eq!(added.source_app_name, None);
        assert_eq!(added.source_app_icon, None);
        assert!(!format!("{added:?}").contains("park me"));
        assert_eq!(session.list_queue(false).expect("list").len(), 1);
        session
            .apply_action(&added.id, "complete")
            .expect("complete");
        assert_eq!(session.list_queue(false).expect("done")[0].status, "done");
        assert!(session.list_overview().expect("overview").is_empty());
        assert!(session.add_composer(String::new()).is_err());
        let edited = session
            .edit_item(&added.id, "Thanks.\nThe persist timeout is the bug.")
            .expect("edit");
        let edited_title = edited.title.expect("edited title");
        assert_eq!(
            edited_title,
            bronze_domain::compact_title("Thanks.\nThe persist timeout is the bug.")
        );
        assert!(edited_title.to_ascii_lowercase().contains("persist"));
        assert!(!edited_title.to_ascii_lowercase().starts_with("thanks"));
    }

    #[test]
    fn live_title_path_writes_compact_title_first() {
        let src = include_str!("live_session.rs");
        let prod = src.split("mod live_session_tests").next().expect("prod");
        let add = prod
            .split("pub fn add_composer(")
            .nth(1)
            .expect("add_composer");
        let add_end = add.find("\n    pub fn ").unwrap_or(add.len());
        assert!(!add[..add_end].contains("refine_title"));
        assert!(!add[..add_end].contains("set_item_title"));
        assert!(!prod.contains("native_item_title"));
        assert!(!prod.contains("bronze-item-title"));
        let body = "Thanks for the note.\nThe migration timeout is the real bug in persist.\nPlease take a look when you can.";
        let mut session = open_session();
        let added = session.add_composer(body.into()).expect("add");
        let title = added.title.expect("title");
        assert_eq!(title, bronze_domain::compact_title(body));
        assert!(title.to_ascii_lowercase().contains("migration"));
        assert!(!title.to_ascii_lowercase().starts_with("thanks"));
        assert!(session
            .apply_refined_title(&added.id, body, "Migration timeout")
            .expect("apply"));
        let stored = session
            .list_queue(true)
            .expect("list")
            .into_iter()
            .find(|item| item.id == added.id)
            .expect("row");
        assert_eq!(stored.title.as_deref(), Some("Migration timeout"));
        assert!(!session
            .apply_refined_title(&added.id, "changed", "Ignored")
            .expect("stale"));
    }

    #[test]
    fn title_model_auto_pick_does_not_overwrite_stored_choice() {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("bronze-title-pick-{n}-{}", now_ms()));
        let first = LiveSession::open(dir.clone()).expect("first");
        assert!(!first.settings().general.title_model.is_unset());
        let mut settings = first.settings();
        settings.general.title_model = TitleModelId::Extractive;
        drop(first);
        let mut writer = LiveSession::open(dir.clone()).expect("writer");
        writer.replace_settings(settings).expect("save extractive");
        drop(writer);
        let again = LiveSession::open(dir).expect("again");
        assert_eq!(
            again.settings().general.title_model,
            TitleModelId::Extractive
        );
        let rows = list_title_models();
        assert!(rows.iter().any(|row| row.id == "extractive" && row.present));
        assert!(rows.iter().any(|row| row.id == "qwen-05"
            && row.filename == "qwen2.5-0.5b-instruct-q4_k_m.gguf"
            && row.vendor_command.contains("qwen-05")));
        assert!(include_str!("live_session.rs").contains("request_tier"));
        assert!(include_str!("live_session.rs").contains("resolve_title_model"));
    }

    #[test]
    fn launch_at_login_persists_without_claiming_os_success() {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("bronze-login-item-{n}-{}", now_ms()));
        let mut session = LiveSession::open(dir.clone()).expect("open");
        let mut settings = session.settings();
        assert!(!settings.general.launch_at_login);
        settings.general.launch_at_login = true;
        session.replace_settings(settings).expect("save");
        assert!(session.settings().general.launch_at_login);
        drop(session);
        let mut again = LiveSession::open(dir).expect("reopen");
        assert!(again.settings().general.launch_at_login);
        again.reset_field("general.launchAtLogin").expect("reset");
        assert!(!again.settings().general.launch_at_login);
        let src = include_str!("live_session.rs");
        assert!(src.contains("apply_login_item"));
        assert!(src.contains("login_item_status"));
        assert!(src.contains("launch_at_login"));
        let lib = include_str!("lib.rs");
        assert!(lib.contains("login_item_status"));
        let tap = include_str!(
            "../../../../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        );
        assert!(!tap.contains("SMAppService"));
        assert!(!tap.contains("bronze_native_apply_login_item"));
        assert_eq!(ADR_018_STATUS, "Proposed");
    }

    #[test]
    fn copy_uses_profile_and_search_stays_placeholder() {
        assert!(!QUE_007_COMPLETE);
        assert_eq!(ADR_018_STATUS, "Proposed");
        let mut session = open_session();
        let first = session.add_composer("Café token".into()).expect("a");
        session.add_composer("plain".into()).expect("b");
        let mut board = FakePasteboard::default();
        let text = session
            .copy_items(&[first.id.clone()], "plain", &mut board)
            .expect("copy");
        assert_eq!(text, "Café token");
        assert_eq!(board.last.as_deref(), Some("Café token"));
        let html = board.last_html.as_deref().expect("html");
        assert!(html.contains("white-space:pre-wrap"));
        assert!(html.contains("Café token"));
        assert!(!html.contains("<script"));
        let (hits, count) = session.search_library("Café").expect("search");
        assert_eq!(count, 1);
        assert_eq!(hits[0].id, first.id);
        assert!(session.search_library("cafe").expect("ascii").0.is_empty());
    }

    #[test]
    fn settings_and_portability_reject_webview_paths() {
        let mut session = open_session();
        session.add_composer("keep".into()).expect("add");
        let mut settings = session.settings();
        settings.data.backup_schedule = bronze_settings::BackupSchedule::Weekly;
        session.replace_settings(settings).expect("save");
        assert_eq!(
            session.settings().data.backup_schedule,
            bronze_settings::BackupSchedule::Weekly
        );
        session.reset_field("data.backupSchedule").expect("reset");
        assert_eq!(
            session.settings().data.backup_schedule,
            bronze_settings::BackupSchedule::Daily
        );
        let backup = session.backup_now().expect("backup");
        assert!(backup.contains("backups"));
        let exported = session.export_library().expect("export");
        assert!(exported.contains("exports"));
        assert_eq!(
            session.import_library("../etc").unwrap_err(),
            "webview_path_rejected"
        );
        assert!(reject_webview_path(Some("/tmp/out".into())).is_err());
        let shortcuts = session.list_shortcuts();
        assert_eq!(
            shortcuts.len(),
            bronze_settings::ShortcutActionId::ALL.len()
        );
        let toggle = shortcuts
            .iter()
            .find(|row| row.action == "app.togglePanel")
            .expect("toggle");
        assert!(toggle.is_default);
        assert_eq!(toggle.logical_key.as_deref(), Some(" "));
        assert_eq!(toggle.scope, "global");
        assert_eq!(toggle.tap_count, None);
        let capture = shortcuts
            .iter()
            .find(|row| row.action == "capture.selection")
            .expect("capture");
        assert_eq!(capture.trigger, "modifier_double_tap");
        assert_eq!(capture.tap_count, Some(2));
        let recorded = session
            .record_shortcut(ShortcutRecordInput {
                action: "queue.search".into(),
                trigger: "accelerator".into(),
                modifiers: vec!["Command".into()],
                logical_key: Some("k".into()),
                tap_count: None,
                skip_test: true,
            })
            .expect("record");
        let search = recorded
            .iter()
            .find(|row| row.action == "queue.search")
            .expect("search");
        assert!(!search.is_default);
        assert_eq!(search.tested.as_deref(), Some("skipped"));
        assert!(
            session
                .restore_shortcut("queue.search")
                .expect("restore")
                .iter()
                .find(|row| row.action == "queue.search")
                .expect("search default")
                .is_default
        );
        assert_eq!(
            session
                .record_shortcut(ShortcutRecordInput {
                    action: "queue.search".into(),
                    trigger: "accelerator".into(),
                    modifiers: vec!["Command".into()],
                    logical_key: Some("../etc".into()),
                    tap_count: None,
                    skip_test: false,
                })
                .unwrap_err(),
            "invalid_shortcut_key"
        );
        assert_eq!(
            session
                .record_shortcut(ShortcutRecordInput {
                    action: "queue.search".into(),
                    trigger: "accelerator".into(),
                    modifiers: vec!["Command".into()],
                    logical_key: Some("c".into()),
                    tap_count: None,
                    skip_test: false,
                })
                .unwrap_err(),
            "shortcut_duplicate"
        );
        assert_eq!(
            session.settings().capture.standard_chord.action,
            bronze_settings::ShortcutActionId::CaptureSelection
        );
        let snapshot = session
            .latest_export_snapshot()
            .expect("name")
            .expect("dir");
        session.import_library(&snapshot).expect("import");
        assert_eq!(effective_ui_locale(Some("system")), "en");
        assert_eq!(effective_ui_locale(Some("en-XA")), "en-XA");
        assert_eq!(effective_ui_locale(Some("nl")), "nl");
        assert_eq!(effective_ui_locale(Some("fr")), "fr");
        assert_eq!(effective_ui_locale(Some("de")), "de");
        assert_eq!(effective_ui_locale(Some("es")), "es");
        assert_eq!(effective_ui_locale(Some("it")), "it");
        assert_eq!(effective_ui_locale(Some("nl-BE")), "nl");
        assert_eq!(effective_ui_locale(Some("zz")), "en");
        assert!(!search_settings("backup").is_empty());
    }

    #[test]
    fn settings_export_import_round_trip_rejects_webview_and_keeps_adr_018() {
        let mut session = open_session();
        let mut settings = session.settings();
        settings.general.locale = "nl".into();
        settings
            .privacy
            .excluded_bundle_ids
            .push("com.bank.app".into());
        session.replace_settings(settings).expect("save");
        session
            .record_shortcut(ShortcutRecordInput {
                action: "queue.search".into(),
                trigger: "accelerator".into(),
                modifiers: vec!["Command".into()],
                logical_key: Some("k".into()),
                tap_count: None,
                skip_test: true,
            })
            .expect("record");
        let preview = session.preview_settings_export().expect("preview");
        assert_eq!(preview.format, "bronze-settings");
        assert_eq!(preview.version, 1);
        assert!(preview
            .sensitive_literal_keys
            .contains(&"privacy.excludedBundleIds".into()));
        assert!(preview
            .sensitive_literal_keys
            .contains(&"shortcuts.queue.search".into()));
        assert!(preview.included_categories.contains(&"privacy".into()));
        let dest = std::env::temp_dir().join(format!(
            "bronze-settings-roundtrip-{}-{}.json",
            now_ms(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let name = session.export_settings_to_path(&dest).expect("export");
        assert_eq!(name, dest.file_name().unwrap().to_string_lossy());
        let raw = fs::read_to_string(&dest).expect("read");
        assert!(raw.contains("bronze-settings"));
        assert!(raw.contains("com.bank.app"));
        assert!(raw.contains("\"nl\""));
        assert!(!raw.contains("permissionToken"));
        assert!(!raw.contains("/Users/"));
        let mut other = open_session();
        other.import_settings_from_path(&dest).expect("import");
        assert_eq!(other.settings().general.locale, "nl");
        assert_eq!(
            other.settings().privacy.excluded_bundle_ids,
            vec!["com.bank.app"]
        );
        let search = other
            .list_shortcuts()
            .into_iter()
            .find(|row| row.action == "queue.search")
            .expect("search");
        assert!(!search.is_default);
        assert_eq!(search.logical_key.as_deref(), Some("k"));
        assert_eq!(ADR_018_STATUS, "Proposed");
        assert!(!QUE_007_COMPLETE);
        assert!(other.search_library("cafe").expect("ascii").0.is_empty());
        assert_eq!(
            session
                .import_settings_from_path(Path::new("../etc/settings.json"))
                .unwrap_err(),
            "settings_path_invalid"
        );
        fs::write(
            &dest,
            r#"{"format":"bronze-export","version":1,"settings":{}}"#,
        )
        .expect("archive");
        assert_eq!(
            session.import_settings_from_path(&dest).unwrap_err(),
            "settings_wrong_format"
        );
        fs::write(&dest, r#"{"format":"bronze-settings","version":2}"#).expect("version");
        assert_eq!(
            session.import_settings_from_path(&dest).unwrap_err(),
            "settings_unknown_version"
        );
        assert!(reject_webview_path(Some("/tmp/settings.json".into())).is_err());
        let used = include_str!("../permissions/used-permissions.toml");
        assert!(used.contains("preview_settings_export"));
        assert!(used.contains("export_settings_file"));
        assert!(used.contains("import_settings_file"));
        assert!(used.contains("list_title_models"));
        assert!(used.contains("login_item_status"));
        assert!(used.contains("preview_support_bundle"));
        assert!(used.contains("export_support_file"));
        let src = include_str!("live_session.rs");
        assert!(src.contains("pub fn preview_settings_export"));
        assert!(src.contains("pub fn export_settings_file"));
        assert!(src.contains("pub fn import_settings_file"));
        assert!(src.contains("pub fn list_title_models"));
        assert!(src.contains("pub fn login_item_status"));
        assert!(src.contains("pub fn preview_support_bundle"));
        assert!(src.contains("pub fn export_support_file"));
        let _ = fs::remove_file(&dest);
    }

    #[test]
    fn support_export_writes_redacted_txt_without_webview_path() {
        use bronze_capture::{AxRole, FakeAnnouncer, FakeAxNode, FakeAxTree, FakeSelection};
        let mut session = open_session();
        let host = FakeSelectionHost {
            tree: FakeAxTree {
                nodes: vec![FakeAxNode::new(
                    AxRole::TextArea,
                    None,
                    FakeSelection::Text("  captured  ".into()),
                    None,
                )],
                focused: Some(0),
                excluded: false,
                accessibility_granted: true,
            },
            source_app_name: Some("TextEdit".into()),
            source_bundle_id: Some("com.apple.TextEdit".into()),
        };
        let mut announce = FakeAnnouncer::default();
        let persisted = session
            .persist_selection(&host, &mut announce, false)
            .expect("persist");
        assert_eq!(persisted.reason, "ok");
        let mut board = FakePasteboard::default();
        let copied = session
            .copy_items(
                &[persisted.item_id.clone().expect("id")],
                "plain",
                &mut board,
            )
            .expect("copy");
        assert_eq!(copied, "  captured  ");
        let preview = session.preview_support_text().expect("preview");
        assert!(!preview.contains("events=0"));
        for needle in [
            "version=",
            "os=",
            "arch=",
            "schema=",
            "path=",
            "input_monitoring=",
            "human_gates=",
            "Last-hour diagnostic events",
            "Last-hour copy attempts",
            "3.9",
        ] {
            assert!(preview.contains(needle), "missing {needle}");
        }
        assert!(!preview.contains("http"));
        assert!(!preview.contains("hunter2"));
        assert!(preview.contains("stage=store"));
        assert!(preview.contains("stage=clipboard"));
        assert!(preview.contains("com.apple.TextEdit"));
        assert!(!preview.contains("captured"));
        let dest = std::env::temp_dir().join(format!(
            "bronze-support-{}.txt",
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let name = session.export_support_to_path(&dest).expect("export");
        assert_eq!(name, dest.file_name().unwrap().to_str().unwrap());
        let body = fs::read_to_string(&dest).expect("read");
        let strip_clock = |text: &str| {
            text.lines()
                .filter(|line| !line.starts_with("generated_at_ms="))
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert_eq!(strip_clock(&preview), strip_clock(&body));
        assert!(body.contains("3.9"));
        assert!(!body.contains("hunter2"));
        assert!(!body.contains("http"));
        assert!(reject_webview_path(Some("/tmp/bronze-support.txt".into())).is_err());
        assert_eq!(
            session
                .export_support_to_path(Path::new("/tmp/bronze-support.json"))
                .unwrap_err(),
            "support_path_invalid"
        );
        let src = include_str!("live_session.rs");
        assert!(src.contains("record_terminal_diag"));
        assert!(src.contains("insert_diagnostic_event"));
        let tap = include_str!(
            "../../../../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        );
        assert!(!tap.contains("insert_diagnostic_event"));
        assert!(!tap.contains("preview_support"));
        assert!(!tap.contains("export_support"));
        assert!(!tap.contains("render_support_report"));
        assert!(!tap.contains("SMAppService"));
        let _ = fs::remove_file(&dest);
    }

    #[test]
    fn persist_nl_exposes_locale_and_english_fallback_catalog() {
        let mut session = open_session();
        assert_eq!(session.ui_locale(), "en");
        let mut settings = session.settings();
        settings.general.locale = "nl".into();
        session.replace_settings(settings).expect("save nl");
        assert_eq!(session.ui_locale(), "nl");
        let catalog = session.ui_catalog();
        assert_eq!(catalog.locale, "nl");
        assert_eq!(catalog.dir, "ltr");
        assert!(catalog.messages.contains_key("settings.title"));
        if catalog.catalog_available {
            assert_eq!(catalog.html_lang, "nl");
        } else {
            assert_eq!(catalog.html_lang, "en");
            assert_eq!(
                catalog.messages.get("settings.title").map(String::as_str),
                Some("Settings")
            );
        }
        session.reset_field("general.locale").expect("reset");
        assert_eq!(session.ui_locale(), "en");
        assert!(session.ui_catalog().catalog_available);
    }

    #[test]
    fn record_shortcut_accepts_modifier_double_tap_and_rejects_duplicate() {
        let mut session = open_session();
        assert_eq!(
            session
                .record_shortcut(ShortcutRecordInput {
                    action: "queue.search".into(),
                    trigger: "modifier_double_tap".into(),
                    modifiers: vec!["Shift".into()],
                    logical_key: None,
                    tap_count: None,
                    skip_test: true,
                })
                .unwrap_err(),
            "shortcut_duplicate"
        );
        let option_rows = session
            .record_shortcut(ShortcutRecordInput {
                action: "queue.search".into(),
                trigger: "modifier_double_tap".into(),
                modifiers: vec!["Option".into()],
                logical_key: Some("o".into()),
                tap_count: None,
                skip_test: true,
            })
            .expect("option double-tap");
        let search = option_rows
            .iter()
            .find(|row| row.action == "queue.search")
            .expect("search");
        assert_eq!(search.trigger, "modifier_double_tap");
        assert_eq!(search.modifiers, vec!["Option".to_string()]);
        assert_eq!(search.logical_key, None);
        assert_eq!(search.tap_count, Some(2));
        assert!(!search.is_default);
        assert_eq!(
            session
                .record_shortcut(ShortcutRecordInput {
                    action: "app.togglePanel".into(),
                    trigger: "modifier_double_tap".into(),
                    modifiers: vec!["Option".into()],
                    logical_key: None,
                    tap_count: None,
                    skip_test: true,
                })
                .unwrap_err(),
            "shortcut_duplicate"
        );
        session
            .record_shortcut(ShortcutRecordInput {
                action: "capture.selection".into(),
                trigger: "accelerator".into(),
                modifiers: vec!["Command".into()],
                logical_key: Some("k".into()),
                tap_count: None,
                skip_test: true,
            })
            .expect("move capture off shift double-tap");
        let shift_rows = session
            .record_shortcut(ShortcutRecordInput {
                action: "queue.complete".into(),
                trigger: "modifier_double_tap".into(),
                modifiers: vec!["Shift".into()],
                logical_key: None,
                tap_count: None,
                skip_test: true,
            })
            .expect("shift double-tap on complete");
        let complete = shift_rows
            .iter()
            .find(|row| row.action == "queue.complete")
            .expect("complete");
        assert_eq!(complete.trigger, "modifier_double_tap");
        assert_eq!(complete.modifiers, vec!["Shift".to_string()]);
        assert_eq!(complete.tap_count, Some(2));
        assert!(!complete.is_default);
        assert_eq!(
            session
                .record_shortcut(ShortcutRecordInput {
                    action: "queue.edit".into(),
                    trigger: "modifier_double_tap".into(),
                    modifiers: Vec::new(),
                    logical_key: None,
                    tap_count: None,
                    skip_test: true,
                })
                .unwrap_err(),
            "shortcut_rejected"
        );
    }

    #[test]
    fn record_shortcut_persists_option_triple_as_distinct_from_double() {
        let mut session = open_session();
        let triple_rows = session
            .record_shortcut(ShortcutRecordInput {
                action: "queue.search".into(),
                trigger: "modifier_double_tap".into(),
                modifiers: vec!["Option".into()],
                logical_key: None,
                tap_count: Some(3),
                skip_test: true,
            })
            .expect("option triple");
        let search = triple_rows
            .iter()
            .find(|row| row.action == "queue.search")
            .expect("search");
        assert_eq!(search.trigger, "modifier_double_tap");
        assert_eq!(search.modifiers, vec!["Option".to_string()]);
        assert_eq!(search.tap_count, Some(3));
        session
            .record_shortcut(ShortcutRecordInput {
                action: "app.togglePanel".into(),
                trigger: "modifier_double_tap".into(),
                modifiers: vec!["Option".into()],
                logical_key: None,
                tap_count: None,
                skip_test: true,
            })
            .expect("option double distinct from triple");
        assert_eq!(
            session
                .record_shortcut(ShortcutRecordInput {
                    action: "queue.complete".into(),
                    trigger: "modifier_double_tap".into(),
                    modifiers: vec!["Option".into()],
                    logical_key: None,
                    tap_count: Some(3),
                    skip_test: true,
                })
                .unwrap_err(),
            "shortcut_duplicate"
        );
        assert_eq!(
            session
                .record_shortcut(ShortcutRecordInput {
                    action: "queue.edit".into(),
                    trigger: "modifier_double_tap".into(),
                    modifiers: vec!["Option".into()],
                    logical_key: None,
                    tap_count: Some(1),
                    skip_test: true,
                })
                .unwrap_err(),
            "invalid_shortcut_tap_count"
        );
        assert_eq!(
            session
                .record_shortcut(ShortcutRecordInput {
                    action: "queue.edit".into(),
                    trigger: "modifier_double_tap".into(),
                    modifiers: vec!["Option".into()],
                    logical_key: None,
                    tap_count: Some(9),
                    skip_test: true,
                })
                .unwrap_err(),
            "invalid_shortcut_tap_count"
        );
    }

    #[test]
    fn ax_capture_persists_before_announce_and_hides_done() {
        use bronze_capture::{
            AxRole, FakeAnnouncer, FakeAxNode, FakeAxTree, FakeSelection, CAPTURE_ONLY_ANNOUNCE_KEY,
        };
        let mut session = open_session();
        let host = FakeSelectionHost {
            tree: FakeAxTree {
                nodes: vec![FakeAxNode::new(
                    AxRole::TextArea,
                    None,
                    FakeSelection::Text("  captured  ".into()),
                    None,
                )],
                focused: Some(0),
                excluded: false,
                accessibility_granted: true,
            },
            source_app_name: Some("TextEdit".into()),
            source_bundle_id: Some("com.apple.TextEdit".into()),
        };
        let mut announce = FakeAnnouncer::default();
        let persisted = session
            .persist_selection(&host, &mut announce, false)
            .expect("persist");
        assert_eq!(persisted.terminal, Terminal::Saved);
        assert_eq!(persisted.reason, "ok");
        let overview = session.list_overview().expect("overview");
        assert_eq!(overview.len(), 1);
        assert_eq!(overview[0].body, "  captured  ");
        assert_eq!(
            overview[0].title,
            Some(bronze_domain::compact_title("  captured  "))
        );
        assert_eq!(overview[0].source_app_name.as_deref(), Some("TextEdit"));
        if let Some(icon) = &overview[0].source_app_icon {
            assert!(icon.starts_with("data:image/png;base64,"));
            assert!(!icon.contains("http://"));
            assert!(!icon.contains("https://"));
        }
        assert_eq!(announce.keys, vec![CAPTURE_ONLY_ANNOUNCE_KEY.to_string()]);
        session
            .apply_action(&overview[0].id, "complete")
            .expect("complete");
        assert!(session.list_overview().expect("hidden").is_empty());
    }

    #[test]
    fn ax_capture_stores_extractive_compact_title() {
        use bronze_capture::{AxRole, FakeAnnouncer, FakeAxNode, FakeAxTree, FakeSelection};
        let body = "Thanks for the note.\nThe migration timeout is the real bug in persist.";
        let mut session = open_session();
        let host = FakeSelectionHost {
            tree: FakeAxTree {
                nodes: vec![FakeAxNode::new(
                    AxRole::TextArea,
                    None,
                    FakeSelection::Text(body.into()),
                    None,
                )],
                focused: Some(0),
                excluded: false,
                accessibility_granted: true,
            },
            source_app_name: Some("TextEdit".into()),
            source_bundle_id: Some("com.apple.TextEdit".into()),
        };
        let mut announce = FakeAnnouncer::default();
        let persisted = session
            .persist_selection(&host, &mut announce, false)
            .expect("persist");
        assert_eq!(persisted.terminal, Terminal::Saved);
        let overview = session.list_overview().expect("overview");
        let title = overview[0].title.clone().expect("title");
        assert_eq!(title, bronze_domain::compact_title(body));
        assert!(title.to_ascii_lowercase().contains("migration"));
        assert!(!title.to_ascii_lowercase().starts_with("thanks"));
    }

    #[test]
    fn ax_capture_rejects_empty_and_secure_without_store() {
        use bronze_capture::{
            AxRole, AxSubrole, FakeAnnouncer, FakeAxNode, FakeAxTree, FakeSelection,
        };
        let mut session = open_session();
        let empty = FakeSelectionHost {
            tree: FakeAxTree {
                nodes: vec![FakeAxNode::new(
                    AxRole::TextField,
                    None,
                    FakeSelection::Empty,
                    None,
                )],
                focused: Some(0),
                excluded: false,
                accessibility_granted: true,
            },
            source_app_name: None,
            source_bundle_id: None,
        };
        let mut announce = FakeAnnouncer::default();
        let terminal = session
            .persist_selection(&empty, &mut announce, false)
            .expect("empty");
        assert_eq!(terminal.terminal, Terminal::Rejected);
        assert_eq!(terminal.reason, "no_selection");
        assert!(session.list_overview().expect("none").is_empty());
        assert!(announce.keys.is_empty());

        let secret = FakeSelectionHost {
            tree: FakeAxTree {
                nodes: vec![FakeAxNode::new(
                    AxRole::TextField,
                    Some(AxSubrole::SecureTextField),
                    FakeSelection::Text("hunter2-secret".into()),
                    None,
                )],
                focused: Some(0),
                excluded: false,
                accessibility_granted: true,
            },
            source_app_name: Some("TextEdit".into()),
            source_bundle_id: None,
        };
        let terminal = session
            .persist_selection(&secret, &mut announce, false)
            .expect("secure");
        assert_eq!(terminal.terminal, Terminal::Rejected);
        assert_eq!(terminal.reason, "protected");
        assert!(session.list_overview().expect("still none").is_empty());
        assert!(!format!("{terminal:?}").contains("hunter2"));
    }

    #[test]
    fn excluded_bundle_rejects_before_ax_read() {
        use bronze_capture::{
            AxRole, FakeAnnouncer, FakeAxNode, FakeAxTree, FakeSelection, Terminal,
        };
        let mut session = open_session();
        let mut settings = session.settings();
        settings.privacy.excluded_bundle_ids =
            vec!["com.secret.app".into(), "com.other.app".into()];
        session.replace_settings(settings).expect("exclude");
        let host = FakeSelectionHost {
            tree: FakeAxTree {
                nodes: vec![FakeAxNode::new(
                    AxRole::TextArea,
                    None,
                    FakeSelection::Text("must-not-persist".into()),
                    None,
                )],
                focused: Some(0),
                excluded: false,
                accessibility_granted: true,
            },
            source_app_name: Some("Secret".into()),
            source_bundle_id: Some("com.secret.app".into()),
        };
        let mut announce = FakeAnnouncer::default();
        let terminal = session
            .persist_selection(&host, &mut announce, false)
            .expect("excluded");
        assert_eq!(terminal.terminal, Terminal::Rejected);
        assert_eq!(terminal.reason, "app_excluded");
        assert!(session.list_overview().expect("none").is_empty());
        assert_eq!(host.tree.total_queries(), 0);
        assert!(announce.keys.is_empty());
        assert_eq!(list_installed_apps().unwrap_err(), "apps_unavailable");
        assert_eq!(
            app_icon_data_url("/Applications/X.app".into()).unwrap_err(),
            "invalid_bundle_id"
        );
        assert_eq!(
            app_icon_data_url("com.apple.Safari".into()).expect("icon"),
            None
        );
        let src = include_str!("live_session.rs");
        assert!(src.contains("peek_bundle_id"));
        assert!(src.contains("app_excluded"));
        assert!(src.contains("apply_live_shift_gesture"));
        assert!(src.contains("apply_live_capture_gesture"));
        assert!(src.contains("bronze_is_frontmost"));
        let from_self = session
            .add_captured("from bronze".into(), Some("Bronze".into()), None)
            .expect("self capture");
        assert_eq!(from_self.source_app_name.as_deref(), Some("Bronze"));
        let own_host = LiveCaptureHost {
            own: Some(crate::own_selection::OwnSelection::Text(
                "inside bronze".into(),
            )),
        };
        let (own_outcome, own_text) = own_host.read();
        assert!(matches!(own_outcome, AxOutcome::Captured { .. }));
        let own_captured = own_text.expect("own");
        assert_eq!(own_captured.text, "inside bronze");
        assert_eq!(own_captured.source_app_name.as_deref(), Some("Bronze"));
        assert!(!format!("{own_captured:?}").contains("inside"));
        let too_large = LiveCaptureHost {
            own: Some(crate::own_selection::OwnSelection::TooLarge),
        };
        assert!(matches!(too_large.read().0, AxOutcome::SelectionTooLarge));
        let fallback = LiveCaptureHost { own: None };
        let _ = fallback.peek_bundle_id();
        let used = include_str!("../permissions/used-permissions.toml");
        assert!(used.contains("list_shortcuts"));
        assert!(used.contains("record_shortcut"));
        assert!(used.contains("restore_shortcut"));
        assert!(used.contains("list_installed_apps"));
        assert!(used.contains("pick_installed_app"));
        assert!(used.contains("app_icon_data_url"));
        assert!(used.contains("preview_settings_export"));
        assert!(used.contains("export_settings_file"));
        assert!(used.contains("import_settings_file"));
        assert!(used.contains("list_title_models"));
        assert!(used.contains("title_engine_status"));
        assert!(used.contains("login_item_status"));
        assert!(used.contains("preview_support_bundle"));
        assert!(used.contains("export_support_file"));
        assert!(src.contains("pub fn pick_installed_app()"));
        assert!(src.contains("picker_cancelled"));
        assert!(src.contains("picker_unavailable"));
    }

    #[test]
    fn own_webview_selection_persists_as_bronze() {
        use bronze_capture::FakeAnnouncer;
        let mut session = open_session();
        let host = LiveCaptureHost {
            own: Some(crate::own_selection::OwnSelection::Text(
                "from the queue".into(),
            )),
        };
        let mut announce = FakeAnnouncer::default();
        let persisted = session
            .persist_selection(&host, &mut announce, false)
            .expect("persist own");
        assert_eq!(persisted.terminal, Terminal::Saved);
        assert_eq!(persisted.reason, "ok");
        let overview = session.list_overview().expect("overview");
        assert_eq!(overview.len(), 1);
        assert_eq!(overview[0].body, "from the queue");
        assert_eq!(overview[0].source_app_name.as_deref(), Some("Bronze"));
    }

    #[test]
    fn capture_result_dto_maps_terminal_without_body() {
        let dto = CaptureResultDto::from_persist(CapturePersistOutcome {
            terminal: Terminal::Rejected,
            reason: "no_selection",
            item_id: None,
        });
        assert_eq!(dto.terminal, "rejected");
        assert_eq!(dto.reason, "no_selection");
        assert_eq!(
            capture_notice_catalog_key("saved", "ok"),
            "capture.announce.saved"
        );
        assert_eq!(
            capture_notice_catalog_key("rejected", "app_excluded"),
            "capture.announce.excluded"
        );
        assert_eq!(
            capture_notice_catalog_key("rejected", "no_selection"),
            "capture.announce.rejected"
        );
        assert_eq!(
            capture_notice_catalog_key("rejected", "accessibility"),
            "capture.announce.denied"
        );
        assert_eq!(
            capture_notice_catalog_key("rejected", "protected"),
            "capture.announce.protected"
        );
        assert_eq!(
            capture_notice_catalog_key("failed", "failed"),
            "capture.announce.failed"
        );
        assert!(!format!("{dto:?}").contains("secret"));
        assert_eq!(encode_base64(b"Man"), "TWFu");
        assert_eq!(encode_base64(b"Ma"), "TWE=");
        assert_eq!(encode_base64(b"M"), "TQ==");
        let tool = format!("{}{}", "pb", "copy");
        assert!(!include_str!("live_session.rs").contains(&tool));
        assert!(!include_str!("copy.rs").contains(&tool));
        let icon = include_str!("live_session.rs");
        assert!(icon.contains("[bundle, name]"));
        assert!(icon.contains("native_app_icon_png(key)"));
    }
}
