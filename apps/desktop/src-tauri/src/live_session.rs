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
use bronze_domain::{
    default_output_profile, ComposerChord, OutputFormat, OutputProfile, PostCopyAction,
};
use bronze_settings::{search_settings, SettingsGroup, SettingsV1};
use bronze_storage::{
    ComposerDraft, ImportStrategy, NoopBackup, Overwrite, PathLocator, QueueAction, QueueItemRow,
    Store, ADR_018_STATUS, QUE_007_COMPLETE,
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

pub fn is_overview_status(status: &str) -> bool {
    matches!(status, "queued" | "copied" | "active")
}

pub trait SelectionHost {
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

impl SelectionHost for LiveAxHost {
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
        let source_bundle_id = bronze_platform_macos::last_external_pid()
            .and_then(bronze_platform_macos::native_bundle_id_for_pid);
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
    data_dir: PathBuf,
}

impl LiveSession {
    pub fn open(data_dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&data_dir).map_err(|err| err.to_string())?;
        let locator = PathLocator {
            path: data_dir.join("bronze.sqlite"),
        };
        let mut backup = NoopBackup;
        let store = Store::open(&locator, &mut backup).map_err(|_| "store_open_failed")?;
        let settings = load_settings_file(&data_dir)?;
        let session = Self {
            store,
            settings,
            data_dir,
        };
        let now = now_ms();
        session
            .store
            .ensure_inbox(now)
            .map_err(|_| "inbox_seed_failed")?;
        Ok(session)
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

    pub fn persist_selection(
        &mut self,
        host: &dyn SelectionHost,
        announcer: &mut dyn Announcer,
        webview_visible: bool,
    ) -> Result<CapturePersistOutcome, String> {
        let (outcome, text) = host.read();
        match (outcome, text) {
            (AxOutcome::Captured { .. }, Some(captured)) => {
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
            (AxOutcome::AccessibilityDenied | AxOutcome::AppExcluded, _) => {
                Ok(CapturePersistOutcome {
                    terminal: Terminal::Rejected,
                    reason: "accessibility",
                    item_id: None,
                })
            }
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

    pub fn set_item_title(&mut self, id: &str, title: &str) -> Result<(), String> {
        self.store
            .set_item_title(id, title, now_ms())
            .map_err(|err| format!("{err:?}"))
    }

    pub fn get_queue_item(&self, id: &str) -> Result<QueueItemDto, String> {
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
        self.settings = next;
        persist_settings_file(&self.data_dir, &self.settings)?;
        Ok(self.settings.clone())
    }

    pub fn reset_field(&mut self, field_id: &str) -> Result<SettingsV1, String> {
        self.settings
            .reset_field(field_id)
            .map_err(|_| "settings_unknown_field")?;
        persist_settings_file(&self.data_dir, &self.settings)?;
        Ok(self.settings.clone())
    }

    pub fn reset_group(&mut self, group: &str) -> Result<SettingsV1, String> {
        let parsed = parse_group(group)?;
        self.settings.reset_group(parsed);
        persist_settings_file(&self.data_dir, &self.settings)?;
        Ok(self.settings.clone())
    }

    pub fn reset_all(&mut self) -> Result<SettingsV1, String> {
        self.settings.reset_all_preserving_content();
        persist_settings_file(&self.data_dir, &self.settings)?;
        Ok(self.settings.clone())
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

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
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
    let item = lock_session(&session)?.add_composer(body)?;
    crate::spawn_title_refine(&app, item.id.clone(), item.body.clone());
    Ok(item)
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
    session: tauri::State<std::sync::Mutex<LiveSession>>,
    id: String,
    body: String,
) -> Result<QueueItemDto, String> {
    lock_session(&session)?.edit_item(&id, &body)
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
        session.set_item_title(&added.id, "Refined").expect("title");
        assert_eq!(
            session
                .get_queue_item(&added.id)
                .expect("get")
                .title
                .as_deref(),
            Some("Refined")
        );
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
    fn capture_result_dto_maps_terminal_without_body() {
        let dto = CaptureResultDto::from_persist(CapturePersistOutcome {
            terminal: Terminal::Rejected,
            reason: "no_selection",
            item_id: None,
        });
        assert_eq!(dto.terminal, "rejected");
        assert_eq!(dto.reason, "no_selection");
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
