//! Typed SettingsV1 + ShortcutActionId (story 7.1, SET-001, WIN-005, docs/12).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_TRASH_RETENTION_DAYS: u32 = 30;
pub const DEFAULT_BACKUP_RETENTION: u32 = 14;
pub const BACKUP_RETENTION_MIN: u32 = 1;
pub const BACKUP_RETENTION_MAX: u32 = 30;
pub const DEFAULT_GAP_MS: u32 = 250;
pub const DEFAULT_MAX_HOLD_MS: u32 = 400;
pub const GAP_MS_MIN: u32 = 150;
pub const GAP_MS_MAX: u32 = 900;
pub const MAX_HOLD_MS_MIN: u32 = 100;
pub const MAX_HOLD_MS_MAX: u32 = 1500;
pub const DEFAULT_DIAGNOSTICS_RETENTION_DAYS: u32 = 7;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ShortcutActionId {
    #[serde(rename = "app.togglePanel")]
    AppTogglePanel,
    #[serde(rename = "capture.selection")]
    CaptureSelection,
    #[serde(rename = "capture.newNote")]
    CaptureNewNote,
    #[serde(rename = "queue.copy")]
    QueueCopy,
    #[serde(rename = "queue.copyWithProfile")]
    QueueCopyWithProfile,
    #[serde(rename = "queue.copyAndAdvance")]
    QueueCopyAndAdvance,
    #[serde(rename = "queue.complete")]
    QueueComplete,
    #[serde(rename = "queue.edit")]
    QueueEdit,
    #[serde(rename = "queue.moveUp")]
    QueueMoveUp,
    #[serde(rename = "queue.moveDown")]
    QueueMoveDown,
    #[serde(rename = "queue.search")]
    QueueSearch,
    #[serde(rename = "queue.undo")]
    QueueUndo,
    #[serde(rename = "window.settings")]
    WindowSettings,
}

impl ShortcutActionId {
    pub const ALL: [Self; 13] = [
        Self::AppTogglePanel,
        Self::CaptureSelection,
        Self::CaptureNewNote,
        Self::QueueCopy,
        Self::QueueCopyWithProfile,
        Self::QueueCopyAndAdvance,
        Self::QueueComplete,
        Self::QueueEdit,
        Self::QueueMoveUp,
        Self::QueueMoveDown,
        Self::QueueSearch,
        Self::QueueUndo,
        Self::WindowSettings,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AppTogglePanel => "app.togglePanel",
            Self::CaptureSelection => "capture.selection",
            Self::CaptureNewNote => "capture.newNote",
            Self::QueueCopy => "queue.copy",
            Self::QueueCopyWithProfile => "queue.copyWithProfile",
            Self::QueueCopyAndAdvance => "queue.copyAndAdvance",
            Self::QueueComplete => "queue.complete",
            Self::QueueEdit => "queue.edit",
            Self::QueueMoveUp => "queue.moveUp",
            Self::QueueMoveDown => "queue.moveDown",
            Self::QueueSearch => "queue.search",
            Self::QueueUndo => "queue.undo",
            Self::WindowSettings => "window.settings",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, SchemaError> {
        Self::ALL
            .iter()
            .copied()
            .find(|id| id.as_str() == raw)
            .ok_or(SchemaError::UnknownShortcutAction)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerKind {
    Accelerator,
    ModifierDoubleTap,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Modifier {
    Command,
    Option,
    Control,
    Shift,
    Fn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum KeyMode {
    Physical,
    Logical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModifierSide {
    Either,
    Left,
    Right,
    Same,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TestedState {
    Tested,
    Untested,
    Skipped,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutBinding {
    pub action: ShortcutActionId,
    pub trigger: TriggerKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<Modifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_mode: Option<KeyMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logical_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier_side: Option<ModifierSide>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_hold_ms: Option<u32>,
    pub enabled: bool,
    pub schema_version: u32,
    pub revision: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tested: Option<TestedState>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModifierTapAction {
    #[serde(rename = "capture.selection")]
    CaptureSelection,
    #[serde(rename = "app.togglePanel")]
    AppTogglePanel,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifierTap {
    pub enabled: bool,
    pub modifier: Modifier,
    pub side: ModifierSide,
    pub action: ModifierTapAction,
    pub gap_ms: u32,
    pub max_hold_ms: u32,
    pub first_tap_feedback: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClipboardFallback {
    Manual,
    SyntheticExperimental,
    Off,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StartView {
    Last,
    ActiveSection,
    Composer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PanelMode {
    Summon,
    Pinned,
    AutoHide,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PanelDisplay {
    Pointer,
    FrontmostApp,
    Last,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PanelEdge {
    Left,
    Right,
    Top,
    LastPosition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Density {
    Comfortable,
    Compact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Translucency {
    System,
    Opaque,
    Material,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceMetadata {
    None,
    App,
    AppAndTitle,
    AppTitleUrl,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InheritAllowDeny {
    Inherit,
    Allow,
    Deny,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InheritAllowDenyAsk {
    Inherit,
    Allow,
    Deny,
    Ask,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InheritProvenance {
    Inherit,
    None,
    App,
    AppAndTitle,
    AppTitleUrl,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPolicy {
    pub capture: InheritAllowDeny,
    pub synthetic_fallback: InheritAllowDenyAsk,
    pub provenance: InheritProvenance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BackupSchedule {
    Daily,
    Weekly,
}

impl BackupSchedule {
    pub fn parse(raw: &str) -> Result<Self, SchemaError> {
        match raw {
            "daily" => Ok(Self::Daily),
            "weekly" => Ok(Self::Weekly),
            "off" | "manual" | "manual-only" | "manualOnly" => {
                Err(SchemaError::BackupScheduleForbidden)
            }
            _ => Err(SchemaError::InvalidBackupSchedule),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContrastPref {
    System,
    More,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DifferentiatePref {
    System,
    Always,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MotionPref {
    System,
    Reduce,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransparencyPref {
    System,
    Reduce,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    pub launch_at_login: bool,
    pub show_dock_icon: bool,
    pub locale: String,
    pub start_view: StartView,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSettings {
    pub standard_chord: ShortcutBinding,
    pub modifier_tap: ModifierTap,
    pub ax_timeout_ms: u32,
    pub clipboard_fallback: ClipboardFallback,
    pub preserve_whitespace: bool,
    pub active_section_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelSettings {
    pub mode: PanelMode,
    pub display: PanelDisplay,
    pub edge: PanelEdge,
    pub width: u32,
    pub density: Density,
    pub always_on_top: bool,
    pub all_spaces: bool,
    pub translucency: Translucency,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopySettings {
    pub default_profile_id: String,
    pub return_to_prior_app: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacySettings {
    pub excluded_bundle_ids: Vec<String>,
    pub source_metadata: SourceMetadata,
    pub app_policies: BTreeMap<String, AppPolicy>,
    pub diagnostics_retention_days: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSettings {
    pub trash_retention_days: u32,
    pub backup_schedule: BackupSchedule,
    pub backup_retention: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilitySettings {
    pub app_scale: u32,
    pub contrast: ContrastPref,
    pub differentiate_without_color: DifferentiatePref,
    pub motion: MotionPref,
    pub transparency: TransparencyPref,
    pub sounds: bool,
    pub haptics: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsV1 {
    pub schema_version: u32,
    pub general: GeneralSettings,
    pub capture: CaptureSettings,
    pub panel: PanelSettings,
    pub copy: CopySettings,
    pub privacy: PrivacySettings,
    pub data: DataSettings,
    pub accessibility: AccessibilitySettings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchemaError {
    UnknownVersion,
    UnknownShortcutAction,
    StandardChordNotCaptureSelection,
    ModifierTapActionForbidden,
    ModifierTapModifierMustBeShift,
    BackupScheduleForbidden,
    InvalidBackupSchedule,
    BackupRetentionOutOfBounds,
    PreserveWhitespaceRequired,
    TimingOutOfBounds,
    UnknownField,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsGroup {
    General,
    Capture,
    Panel,
    Copy,
    Privacy,
    Data,
    Accessibility,
}

impl SettingsGroup {
    pub const ALL: [Self; 7] = [
        Self::General,
        Self::Capture,
        Self::Panel,
        Self::Copy,
        Self::Privacy,
        Self::Data,
        Self::Accessibility,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Capture => "capture",
            Self::Panel => "panel",
            Self::Copy => "copy",
            Self::Privacy => "privacy",
            Self::Data => "data",
            Self::Accessibility => "accessibility",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SettingsField {
    pub id: &'static str,
    pub group: SettingsGroup,
    pub tokens: &'static [&'static str],
}

pub const SETTINGS_FIELDS: &[SettingsField] = &[
    SettingsField {
        id: "general.launchAtLogin",
        group: SettingsGroup::General,
        tokens: &["launch", "login"],
    },
    SettingsField {
        id: "general.showDockIcon",
        group: SettingsGroup::General,
        tokens: &["dock"],
    },
    SettingsField {
        id: "general.locale",
        group: SettingsGroup::General,
        tokens: &["locale", "language"],
    },
    SettingsField {
        id: "general.startView",
        group: SettingsGroup::General,
        tokens: &["start", "view"],
    },
    SettingsField {
        id: "capture.standardChord",
        group: SettingsGroup::Capture,
        tokens: &["shortcut", "chord", "capture"],
    },
    SettingsField {
        id: "capture.modifierTap",
        group: SettingsGroup::Capture,
        tokens: &["double", "tap", "shift"],
    },
    SettingsField {
        id: "capture.axTimeoutMs",
        group: SettingsGroup::Capture,
        tokens: &["timeout", "ax"],
    },
    SettingsField {
        id: "capture.clipboardFallback",
        group: SettingsGroup::Capture,
        tokens: &["clipboard", "fallback"],
    },
    SettingsField {
        id: "capture.preserveWhitespace",
        group: SettingsGroup::Capture,
        tokens: &["whitespace"],
    },
    SettingsField {
        id: "capture.activeSectionId",
        group: SettingsGroup::Capture,
        tokens: &["section"],
    },
    SettingsField {
        id: "panel.mode",
        group: SettingsGroup::Panel,
        tokens: &["summon", "pinned"],
    },
    SettingsField {
        id: "panel.display",
        group: SettingsGroup::Panel,
        tokens: &["display", "pointer"],
    },
    SettingsField {
        id: "panel.edge",
        group: SettingsGroup::Panel,
        tokens: &["edge"],
    },
    SettingsField {
        id: "panel.width",
        group: SettingsGroup::Panel,
        tokens: &["width"],
    },
    SettingsField {
        id: "panel.density",
        group: SettingsGroup::Panel,
        tokens: &["density"],
    },
    SettingsField {
        id: "panel.alwaysOnTop",
        group: SettingsGroup::Panel,
        tokens: &["top"],
    },
    SettingsField {
        id: "panel.allSpaces",
        group: SettingsGroup::Panel,
        tokens: &["spaces"],
    },
    SettingsField {
        id: "panel.translucency",
        group: SettingsGroup::Panel,
        tokens: &["translucency", "opaque"],
    },
    SettingsField {
        id: "copy.defaultProfileId",
        group: SettingsGroup::Copy,
        tokens: &["profile"],
    },
    SettingsField {
        id: "copy.returnToPriorApp",
        group: SettingsGroup::Copy,
        tokens: &["prior", "focus"],
    },
    SettingsField {
        id: "privacy.excludedBundleIds",
        group: SettingsGroup::Privacy,
        tokens: &["exclude", "bundle"],
    },
    SettingsField {
        id: "privacy.sourceMetadata",
        group: SettingsGroup::Privacy,
        tokens: &["provenance", "metadata"],
    },
    SettingsField {
        id: "privacy.appPolicies",
        group: SettingsGroup::Privacy,
        tokens: &["policy", "policies"],
    },
    SettingsField {
        id: "privacy.diagnosticsRetentionDays",
        group: SettingsGroup::Privacy,
        tokens: &["retention"],
    },
    SettingsField {
        id: "data.trashRetentionDays",
        group: SettingsGroup::Data,
        tokens: &["trash"],
    },
    SettingsField {
        id: "data.backupSchedule",
        group: SettingsGroup::Data,
        tokens: &["backup", "daily", "weekly"],
    },
    SettingsField {
        id: "data.backupRetention",
        group: SettingsGroup::Data,
        tokens: &["backup", "retention"],
    },
    SettingsField {
        id: "accessibility.appScale",
        group: SettingsGroup::Accessibility,
        tokens: &["scale"],
    },
    SettingsField {
        id: "accessibility.contrast",
        group: SettingsGroup::Accessibility,
        tokens: &["contrast"],
    },
    SettingsField {
        id: "accessibility.differentiateWithoutColor",
        group: SettingsGroup::Accessibility,
        tokens: &["color"],
    },
    SettingsField {
        id: "accessibility.motion",
        group: SettingsGroup::Accessibility,
        tokens: &["motion"],
    },
    SettingsField {
        id: "accessibility.transparency",
        group: SettingsGroup::Accessibility,
        tokens: &["transparency"],
    },
    SettingsField {
        id: "accessibility.sounds",
        group: SettingsGroup::Accessibility,
        tokens: &["sounds"],
    },
    SettingsField {
        id: "accessibility.haptics",
        group: SettingsGroup::Accessibility,
        tokens: &["haptics"],
    },
];

pub fn default_standard_chord() -> ShortcutBinding {
    ShortcutBinding {
        action: ShortcutActionId::CaptureSelection,
        trigger: TriggerKind::ModifierDoubleTap,
        modifiers: vec![Modifier::Shift],
        key_mode: None,
        physical_code: None,
        logical_key: None,
        modifier_side: Some(ModifierSide::Either),
        gap_ms: Some(DEFAULT_GAP_MS),
        max_hold_ms: Some(DEFAULT_MAX_HOLD_MS),
        enabled: true,
        schema_version: SCHEMA_VERSION,
        revision: 1,
        tested: Some(TestedState::Untested),
    }
}

impl SettingsV1 {
    pub fn defaults() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            general: GeneralSettings {
                launch_at_login: false,
                show_dock_icon: false,
                locale: "system".into(),
                start_view: StartView::Last,
            },
            capture: CaptureSettings {
                standard_chord: default_standard_chord(),
                modifier_tap: ModifierTap {
                    enabled: true,
                    modifier: Modifier::Shift,
                    side: ModifierSide::Either,
                    action: ModifierTapAction::CaptureSelection,
                    gap_ms: DEFAULT_GAP_MS,
                    max_hold_ms: DEFAULT_MAX_HOLD_MS,
                    first_tap_feedback: false,
                },
                ax_timeout_ms: 250,
                clipboard_fallback: ClipboardFallback::Manual,
                preserve_whitespace: true,
                active_section_id: String::new(),
            },
            panel: PanelSettings {
                mode: PanelMode::Summon,
                display: PanelDisplay::FrontmostApp,
                edge: PanelEdge::Right,
                width: 360,
                density: Density::Comfortable,
                always_on_top: false,
                all_spaces: true,
                translucency: Translucency::System,
            },
            copy: CopySettings {
                default_profile_id: "plain".into(),
                return_to_prior_app: true,
            },
            privacy: PrivacySettings {
                excluded_bundle_ids: Vec::new(),
                source_metadata: SourceMetadata::None,
                app_policies: BTreeMap::new(),
                diagnostics_retention_days: DEFAULT_DIAGNOSTICS_RETENTION_DAYS,
            },
            data: DataSettings {
                trash_retention_days: DEFAULT_TRASH_RETENTION_DAYS,
                backup_schedule: BackupSchedule::Daily,
                backup_retention: DEFAULT_BACKUP_RETENTION,
            },
            accessibility: AccessibilitySettings {
                app_scale: 100,
                contrast: ContrastPref::System,
                differentiate_without_color: DifferentiatePref::System,
                motion: MotionPref::System,
                transparency: TransparencyPref::System,
                sounds: false,
                haptics: false,
            },
        }
    }

    pub fn validate(&self) -> Result<(), SchemaError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(SchemaError::UnknownVersion);
        }
        if !self.capture.preserve_whitespace {
            return Err(SchemaError::PreserveWhitespaceRequired);
        }
        if self.capture.standard_chord.action != ShortcutActionId::CaptureSelection {
            return Err(SchemaError::StandardChordNotCaptureSelection);
        }
        if self.capture.modifier_tap.modifier != Modifier::Shift {
            return Err(SchemaError::ModifierTapModifierMustBeShift);
        }
        if !(GAP_MS_MIN..=GAP_MS_MAX).contains(&self.capture.modifier_tap.gap_ms)
            || !(MAX_HOLD_MS_MIN..=MAX_HOLD_MS_MAX).contains(&self.capture.modifier_tap.max_hold_ms)
        {
            return Err(SchemaError::TimingOutOfBounds);
        }
        if !(BACKUP_RETENTION_MIN..=BACKUP_RETENTION_MAX).contains(&self.data.backup_retention) {
            return Err(SchemaError::BackupRetentionOutOfBounds);
        }
        Ok(())
    }

    pub fn from_json(raw: &str) -> Result<Self, SchemaError> {
        let value: serde_json::Value =
            serde_json::from_str(raw).map_err(|_| SchemaError::UnknownVersion)?;
        let version = value.get("schemaVersion").and_then(|v| v.as_u64());
        match version {
            Some(1) => {}
            _ => return Err(SchemaError::UnknownVersion),
        }
        let parsed: Self =
            serde_json::from_value(value).map_err(|_| SchemaError::UnknownVersion)?;
        parsed.validate()?;
        Ok(parsed)
    }

    pub fn to_json(&self) -> Result<String, SchemaError> {
        self.validate()?;
        serde_json::to_string(self).map_err(|_| SchemaError::UnknownVersion)
    }

    pub fn reset_field(&mut self, field_id: &str) -> Result<(), SchemaError> {
        let defaults = Self::defaults();
        match field_id {
            "general.launchAtLogin" => {
                self.general.launch_at_login = defaults.general.launch_at_login
            }
            "general.showDockIcon" => self.general.show_dock_icon = defaults.general.show_dock_icon,
            "general.locale" => self.general.locale = defaults.general.locale,
            "general.startView" => self.general.start_view = defaults.general.start_view,
            "capture.standardChord" => {
                self.capture.standard_chord = defaults.capture.standard_chord
            }
            "capture.modifierTap" => self.capture.modifier_tap = defaults.capture.modifier_tap,
            "capture.axTimeoutMs" => self.capture.ax_timeout_ms = defaults.capture.ax_timeout_ms,
            "capture.clipboardFallback" => {
                self.capture.clipboard_fallback = defaults.capture.clipboard_fallback
            }
            "capture.preserveWhitespace" => {
                self.capture.preserve_whitespace = defaults.capture.preserve_whitespace
            }
            "capture.activeSectionId" => {
                self.capture.active_section_id = defaults.capture.active_section_id
            }
            "panel.mode" => self.panel.mode = defaults.panel.mode,
            "panel.display" => self.panel.display = defaults.panel.display,
            "panel.edge" => self.panel.edge = defaults.panel.edge,
            "panel.width" => self.panel.width = defaults.panel.width,
            "panel.density" => self.panel.density = defaults.panel.density,
            "panel.alwaysOnTop" => self.panel.always_on_top = defaults.panel.always_on_top,
            "panel.allSpaces" => self.panel.all_spaces = defaults.panel.all_spaces,
            "panel.translucency" => self.panel.translucency = defaults.panel.translucency,
            "copy.defaultProfileId" => {
                self.copy.default_profile_id = defaults.copy.default_profile_id
            }
            "copy.returnToPriorApp" => {
                self.copy.return_to_prior_app = defaults.copy.return_to_prior_app
            }
            "privacy.excludedBundleIds" => {
                self.privacy.excluded_bundle_ids = defaults.privacy.excluded_bundle_ids
            }
            "privacy.sourceMetadata" => {
                self.privacy.source_metadata = defaults.privacy.source_metadata
            }
            "privacy.appPolicies" => self.privacy.app_policies = defaults.privacy.app_policies,
            "privacy.diagnosticsRetentionDays" => {
                self.privacy.diagnostics_retention_days =
                    defaults.privacy.diagnostics_retention_days
            }
            "data.trashRetentionDays" => {
                self.data.trash_retention_days = defaults.data.trash_retention_days
            }
            "data.backupSchedule" => self.data.backup_schedule = defaults.data.backup_schedule,
            "data.backupRetention" => self.data.backup_retention = defaults.data.backup_retention,
            "accessibility.appScale" => {
                self.accessibility.app_scale = defaults.accessibility.app_scale
            }
            "accessibility.contrast" => {
                self.accessibility.contrast = defaults.accessibility.contrast
            }
            "accessibility.differentiateWithoutColor" => {
                self.accessibility.differentiate_without_color =
                    defaults.accessibility.differentiate_without_color
            }
            "accessibility.motion" => self.accessibility.motion = defaults.accessibility.motion,
            "accessibility.transparency" => {
                self.accessibility.transparency = defaults.accessibility.transparency
            }
            "accessibility.sounds" => self.accessibility.sounds = defaults.accessibility.sounds,
            "accessibility.haptics" => self.accessibility.haptics = defaults.accessibility.haptics,
            _ => return Err(SchemaError::UnknownField),
        }
        Ok(())
    }

    pub fn reset_group(&mut self, group: SettingsGroup) {
        let defaults = Self::defaults();
        match group {
            SettingsGroup::General => self.general = defaults.general,
            SettingsGroup::Capture => self.capture = defaults.capture,
            SettingsGroup::Panel => self.panel = defaults.panel,
            SettingsGroup::Copy => self.copy = defaults.copy,
            SettingsGroup::Privacy => self.privacy = defaults.privacy,
            SettingsGroup::Data => self.data = defaults.data,
            SettingsGroup::Accessibility => self.accessibility = defaults.accessibility,
        }
    }

    pub fn reset_all_preserving_content(&mut self) {
        *self = Self::defaults();
    }
}

pub fn search_settings(query: &str) -> Vec<&'static SettingsField> {
    let q = query.trim().to_ascii_lowercase();
    SETTINGS_FIELDS
        .iter()
        .filter(|field| {
            if q.is_empty() {
                return true;
            }
            field.id.to_ascii_lowercase().contains(&q)
                || field.group.as_str().contains(q.as_str())
                || field
                    .tokens
                    .iter()
                    .any(|token| token.contains(&q) || q.contains(token))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_defaults_are_valid_and_daily_backup() {
        let settings = SettingsV1::defaults();
        settings.validate().expect("defaults");
        assert_eq!(settings.schema_version, 1);
        assert_eq!(settings.data.backup_schedule, BackupSchedule::Daily);
        assert_eq!(
            settings.data.trash_retention_days,
            DEFAULT_TRASH_RETENTION_DAYS
        );
        assert_eq!(
            settings.capture.standard_chord.action,
            ShortcutActionId::CaptureSelection
        );
        assert!(settings.capture.preserve_whitespace);
        assert_eq!(
            settings.capture.clipboard_fallback,
            ClipboardFallback::Manual
        );
        assert_eq!(settings.privacy.source_metadata, SourceMetadata::None);
        assert!(!settings.general.launch_at_login);
    }

    #[test]
    fn settings_rejects_unknown_schema_version() {
        assert_eq!(
            SettingsV1::from_json(r#"{"schemaVersion":2}"#).unwrap_err(),
            SchemaError::UnknownVersion
        );
        assert_eq!(
            SettingsV1::from_json(r#"{"schemaVersion":0}"#).unwrap_err(),
            SchemaError::UnknownVersion
        );
    }

    #[test]
    fn shortcut_action_ids_match_docs_12() {
        let expected = [
            "app.togglePanel",
            "capture.selection",
            "capture.newNote",
            "queue.copy",
            "queue.copyWithProfile",
            "queue.copyAndAdvance",
            "queue.complete",
            "queue.edit",
            "queue.moveUp",
            "queue.moveDown",
            "queue.search",
            "queue.undo",
            "window.settings",
        ];
        assert_eq!(ShortcutActionId::ALL.len(), expected.len());
        for (id, name) in ShortcutActionId::ALL.iter().zip(expected) {
            assert_eq!(id.as_str(), name);
            assert_eq!(ShortcutActionId::parse(name).unwrap(), *id);
        }
        assert_eq!(
            ShortcutActionId::parse("queue.paste").unwrap_err(),
            SchemaError::UnknownShortcutAction
        );
    }

    #[test]
    fn backup_schedule_rejects_off_and_manual() {
        assert_eq!(
            BackupSchedule::parse("daily").unwrap(),
            BackupSchedule::Daily
        );
        assert_eq!(
            BackupSchedule::parse("weekly").unwrap(),
            BackupSchedule::Weekly
        );
        assert_eq!(
            BackupSchedule::parse("off").unwrap_err(),
            SchemaError::BackupScheduleForbidden
        );
        assert_eq!(
            BackupSchedule::parse("manual").unwrap_err(),
            SchemaError::BackupScheduleForbidden
        );
        assert_eq!(
            BackupSchedule::parse("manual-only").unwrap_err(),
            SchemaError::BackupScheduleForbidden
        );
        let mut raw = SettingsV1::defaults().to_json().unwrap();
        raw = raw.replace("\"daily\"", "\"off\"");
        assert!(SettingsV1::from_json(&raw).is_err());
    }

    #[test]
    fn standard_chord_is_capture_selection_view() {
        let mut settings = SettingsV1::defaults();
        settings.capture.standard_chord.action = ShortcutActionId::QueueCopy;
        assert_eq!(
            settings.validate().unwrap_err(),
            SchemaError::StandardChordNotCaptureSelection
        );
    }

    #[test]
    fn settings_json_round_trip() {
        let original = SettingsV1::defaults();
        let json = original.to_json().expect("json");
        assert!(json.contains("\"schemaVersion\":1"));
        assert!(json.contains("\"backupSchedule\":\"daily\""));
        assert!(json.contains("\"standardChord\""));
        let parsed = SettingsV1::from_json(&json).expect("parse");
        assert_eq!(parsed, original);
    }

    #[test]
    fn search_finds_backup_and_group() {
        let backup = search_settings("backup");
        assert!(backup.iter().any(|field| field.id == "data.backupSchedule"));
        let general = search_settings("general");
        assert!(general
            .iter()
            .all(|field| field.group == SettingsGroup::General));
        assert!(general.iter().any(|field| field.id == "general.locale"));
    }

    #[test]
    fn reset_field_group_and_all() {
        let mut settings = SettingsV1::defaults();
        settings.data.backup_schedule = BackupSchedule::Weekly;
        settings.data.trash_retention_days = 7;
        settings.general.launch_at_login = true;
        settings.reset_field("data.backupSchedule").expect("reset");
        assert_eq!(settings.data.backup_schedule, BackupSchedule::Daily);
        assert_eq!(settings.data.trash_retention_days, 7);
        settings.reset_group(SettingsGroup::Data);
        assert_eq!(
            settings.data.trash_retention_days,
            DEFAULT_TRASH_RETENTION_DAYS
        );
        assert!(settings.general.launch_at_login);
        settings.reset_all_preserving_content();
        assert!(!settings.general.launch_at_login);
        assert_eq!(settings, SettingsV1::defaults());
    }
}
