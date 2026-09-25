//! Shortcut registry and recorder policy (story 7.2, SET-002, CAP-001, CAP-002).

use crate::schema::{
    KeyMode, Modifier, ModifierSide, ShortcutActionId, ShortcutBinding, TestedState, TriggerKind,
    DEFAULT_GAP_MS, DEFAULT_MAX_HOLD_MS, SCHEMA_VERSION,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegisterError {
    NativeRejected,
    Duplicate,
    AlternativesRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShortcutScope {
    Global,
    AppLocal,
}

pub fn shortcut_scope(action: ShortcutActionId) -> ShortcutScope {
    match action {
        ShortcutActionId::AppTogglePanel | ShortcutActionId::CaptureSelection => {
            ShortcutScope::Global
        }
        _ => ShortcutScope::AppLocal,
    }
}

pub fn default_shortcut_binding(action: ShortcutActionId) -> ShortcutBinding {
    match action {
        ShortcutActionId::CaptureSelection => crate::schema::default_standard_chord(),
        ShortcutActionId::AppTogglePanel => accelerator(action, vec![Modifier::Option], " "),
        ShortcutActionId::CaptureNewNote => accelerator(action, vec![Modifier::Command], "n"),
        ShortcutActionId::QueueCopy => accelerator(action, vec![Modifier::Command], "c"),
        ShortcutActionId::QueueCopyWithProfile => {
            accelerator(action, vec![Modifier::Command, Modifier::Shift], "c")
        }
        ShortcutActionId::QueueCopyAndAdvance => {
            accelerator(action, vec![Modifier::Command, Modifier::Shift], "Enter")
        }
        ShortcutActionId::QueueComplete => accelerator(action, Vec::new(), " "),
        ShortcutActionId::QueueEdit => accelerator(action, Vec::new(), "Enter"),
        ShortcutActionId::QueueMoveUp => {
            accelerator(action, vec![Modifier::Option, Modifier::Command], "ArrowUp")
        }
        ShortcutActionId::QueueMoveDown => accelerator(
            action,
            vec![Modifier::Option, Modifier::Command],
            "ArrowDown",
        ),
        ShortcutActionId::QueueSearch => accelerator(action, vec![Modifier::Command], "f"),
        ShortcutActionId::QueueUndo => accelerator(action, vec![Modifier::Command], "z"),
        ShortcutActionId::WindowSettings => accelerator(action, vec![Modifier::Command], ","),
    }
}

pub fn is_default_binding(binding: &ShortcutBinding) -> bool {
    let default = default_shortcut_binding(binding.action);
    same_assigned_chord(binding, &default) && binding.enabled == default.enabled
}

pub const MAX_MODIFIER_TAPS: u32 = 8;

pub fn effective_tap_count(binding: &ShortcutBinding) -> Option<u32> {
    if binding.trigger != TriggerKind::ModifierDoubleTap {
        return None;
    }
    Some(binding.tap_count.unwrap_or(2).clamp(2, MAX_MODIFIER_TAPS))
}

pub fn apply_recorded_double_tap_timing(binding: &mut ShortcutBinding) {
    if binding.trigger != TriggerKind::ModifierDoubleTap {
        binding.gap_ms = None;
        binding.max_hold_ms = None;
        binding.modifier_side = None;
        binding.tap_count = None;
        return;
    }
    binding.logical_key = None;
    binding.key_mode = None;
    binding.physical_code = None;
    if binding.gap_ms.is_none() {
        binding.gap_ms = Some(DEFAULT_GAP_MS);
    }
    if binding.max_hold_ms.is_none() {
        binding.max_hold_ms = Some(DEFAULT_MAX_HOLD_MS);
    }
    if binding.modifier_side.is_none() {
        binding.modifier_side = Some(ModifierSide::Either);
    }
    binding.tap_count = effective_tap_count(binding);
}

pub fn recorded_double_tap_allowed(binding: &ShortcutBinding) -> bool {
    binding.trigger == TriggerKind::ModifierDoubleTap
        && binding.enabled
        && binding.logical_key.is_none()
        && binding.modifiers.len() == 1
        && effective_tap_count(binding).is_some()
}

pub fn live_shift_tap_count(binding: &ShortcutBinding) -> Option<u32> {
    if !binding.enabled || binding.action != ShortcutActionId::CaptureSelection {
        return None;
    }
    if binding.trigger != TriggerKind::ModifierDoubleTap {
        return None;
    }
    if binding.modifiers.as_slice() != [Modifier::Shift] {
        return None;
    }
    effective_tap_count(binding)
}

fn accelerator(action: ShortcutActionId, modifiers: Vec<Modifier>, key: &str) -> ShortcutBinding {
    ShortcutBinding {
        action,
        trigger: TriggerKind::Accelerator,
        modifiers,
        key_mode: Some(KeyMode::Logical),
        physical_code: None,
        logical_key: Some(key.into()),
        modifier_side: None,
        gap_ms: None,
        max_hold_ms: None,
        tap_count: None,
        enabled: true,
        schema_version: SCHEMA_VERSION,
        revision: 1,
        tested: Some(TestedState::Untested),
    }
}

fn is_factory_placeholder(binding: &ShortcutBinding) -> bool {
    binding.revision == 1 && !binding.enabled && binding.trigger == TriggerKind::Disabled
}

pub fn logical_key_allowed(key: &str) -> bool {
    if key.len() > 16 || key.contains('/') || key.contains('\\') || key.contains('\0') {
        return false;
    }
    matches!(
        key,
        " " | "Enter"
            | "Escape"
            | "Tab"
            | "ArrowUp"
            | "ArrowDown"
            | "ArrowLeft"
            | "ArrowRight"
            | "Backspace"
            | "Delete"
            | ","
            | "."
            | ";"
            | "'"
            | "["
            | "]"
            | "`"
            | "-"
            | "="
    ) || (key.len() == 1 && key.chars().all(|ch| ch.is_ascii_alphanumeric()))
        || key
            .strip_prefix('F')
            .and_then(|rest| rest.parse::<u8>().ok())
            .is_some_and(|n| (1..=19).contains(&n))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecorderIgnore {
    ImeComposing,
    VoiceOverReserved,
    KeyRepeat,
    LoneCharacter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecorderEvent {
    pub is_composing: bool,
    pub is_repeat: bool,
    pub is_voiceover_reserved: bool,
    pub has_modifier: bool,
    pub is_character: bool,
    pub is_global: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureAlternatives {
    pub chord: bool,
    pub menu: bool,
    pub manual: bool,
}

pub fn capture_alternatives_ok(alt: CaptureAlternatives) -> bool {
    alt.chord || alt.menu || alt.manual
}

pub fn recorder_swallows(event: RecorderEvent) -> Result<bool, RecorderIgnore> {
    if event.is_composing {
        return Err(RecorderIgnore::ImeComposing);
    }
    if event.is_voiceover_reserved {
        return Err(RecorderIgnore::VoiceOverReserved);
    }
    if event.is_repeat {
        return Err(RecorderIgnore::KeyRepeat);
    }
    if event.is_global && event.is_character && !event.has_modifier {
        return Err(RecorderIgnore::LoneCharacter);
    }
    Ok(true)
}

pub fn skip_test_marks_untested(binding: &mut ShortcutBinding) {
    binding.tested = Some(TestedState::Skipped);
}

pub trait NativeRegistrar {
    fn try_register(&mut self, binding: &ShortcutBinding) -> Result<(), RegisterError>;
}

#[derive(Clone, Debug, Default)]
pub struct FakeRegistrar {
    pub fail: bool,
}

impl NativeRegistrar for FakeRegistrar {
    fn try_register(&mut self, _binding: &ShortcutBinding) -> Result<(), RegisterError> {
        if self.fail {
            Err(RegisterError::NativeRejected)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShortcutRegistry {
    bindings: BTreeMap<ShortcutActionId, ShortcutBinding>,
}

impl ShortcutRegistry {
    pub fn seeded() -> Self {
        let mut bindings = BTreeMap::new();
        for action in ShortcutActionId::ALL {
            bindings.insert(action, default_binding(action));
        }
        Self { bindings }
    }

    pub fn get(&self, action: ShortcutActionId) -> &ShortcutBinding {
        self.bindings
            .get(&action)
            .expect("seeded registry lists every ShortcutActionId")
    }

    pub fn standard_chord(&self) -> &ShortcutBinding {
        self.get(ShortcutActionId::CaptureSelection)
    }

    pub fn all_bindings(&self) -> Vec<ShortcutBinding> {
        ShortcutActionId::ALL
            .iter()
            .copied()
            .map(|action| self.get(action).clone())
            .collect()
    }

    pub fn persist_rows(&self) -> Vec<(String, String, bool, u32)> {
        self.bindings
            .values()
            .map(|binding| {
                let json = serde_json::to_string(binding).expect("binding json");
                (
                    binding.action.as_str().to_string(),
                    json,
                    binding.enabled,
                    binding.revision,
                )
            })
            .collect()
    }

    pub fn from_rows(rows: &[(String, String, bool, u32)]) -> Result<Self, RegisterError> {
        let mut registry = Self::seeded();
        for (action, json, enabled, revision) in rows {
            let action = ShortcutActionId::parse(action).map_err(|_| RegisterError::Duplicate)?;
            let mut binding: ShortcutBinding =
                serde_json::from_str(json).map_err(|_| RegisterError::Duplicate)?;
            binding.action = action;
            binding.enabled = *enabled;
            binding.revision = *revision;
            if is_factory_placeholder(&binding) {
                continue;
            }
            registry.bindings.insert(action, binding);
        }
        if registry.bindings.len() != ShortcutActionId::ALL.len() {
            return Err(RegisterError::Duplicate);
        }
        Ok(registry)
    }

    pub fn register(
        &mut self,
        candidate: ShortcutBinding,
        registrar: &mut impl NativeRegistrar,
        alternatives: CaptureAlternatives,
    ) -> Result<(), RegisterError> {
        if candidate.action == ShortcutActionId::CaptureSelection && !candidate.enabled {
            let remaining = CaptureAlternatives {
                chord: false,
                menu: alternatives.menu,
                manual: alternatives.manual,
            };
            if !capture_alternatives_ok(remaining) {
                return Err(RegisterError::AlternativesRequired);
            }
        }
        if let Some(existing) = self.bindings.values().find(|binding| {
            binding.action != candidate.action
                && binding.enabled
                && candidate.enabled
                && same_chord(binding, &candidate)
        }) {
            let _ = existing;
            return Err(RegisterError::Duplicate);
        }
        let previous = self.get(candidate.action).clone();
        match registrar.try_register(&candidate) {
            Ok(()) => {
                self.bindings.insert(candidate.action, candidate);
                Ok(())
            }
            Err(err) => {
                self.bindings.insert(previous.action, previous);
                Err(err)
            }
        }
    }

    pub fn restore_default(
        &mut self,
        action: ShortcutActionId,
        registrar: &mut impl NativeRegistrar,
        alternatives: CaptureAlternatives,
    ) -> Result<(), RegisterError> {
        self.register(default_shortcut_binding(action), registrar, alternatives)
    }
}

fn default_binding(action: ShortcutActionId) -> ShortcutBinding {
    default_shortcut_binding(action)
}

fn same_chord(left: &ShortcutBinding, right: &ShortcutBinding) -> bool {
    same_assigned_chord(left, right) && left.trigger != TriggerKind::Disabled
}

fn same_assigned_chord(left: &ShortcutBinding, right: &ShortcutBinding) -> bool {
    left.trigger == right.trigger
        && left.modifiers == right.modifiers
        && left.logical_key == right.logical_key
        && left.physical_code == right.physical_code
        && left.modifier_side == right.modifier_side
        && left.gap_ms == right.gap_ms
        && left.max_hold_ms == right.max_hold_ms
        && effective_tap_count(left) == effective_tap_count(right)
}

#[cfg(test)]
mod shortcuts_tests {
    use super::*;
    use crate::schema::{Modifier, ShortcutActionId};

    fn chord(action: ShortcutActionId, key: &str) -> ShortcutBinding {
        ShortcutBinding {
            action,
            trigger: TriggerKind::Accelerator,
            modifiers: vec![Modifier::Command],
            key_mode: None,
            physical_code: None,
            logical_key: Some(key.into()),
            modifier_side: None,
            gap_ms: None,
            max_hold_ms: None,
            tap_count: None,
            enabled: true,
            schema_version: SCHEMA_VERSION,
            revision: 2,
            tested: Some(TestedState::Untested),
        }
    }

    #[test]
    fn shortcuts_all_actions_persist_and_standard_chord_is_capture_selection() {
        let registry = ShortcutRegistry::seeded();
        let rows = registry.persist_rows();
        assert_eq!(rows.len(), ShortcutActionId::ALL.len());
        for action in ShortcutActionId::ALL {
            assert!(rows.iter().any(|(id, _, _, _)| id == action.as_str()));
        }
        assert_eq!(
            registry.standard_chord().action,
            ShortcutActionId::CaptureSelection
        );
        let restored = ShortcutRegistry::from_rows(&rows).expect("rows");
        assert_eq!(restored.persist_rows().len(), ShortcutActionId::ALL.len());
    }

    #[test]
    fn shortcuts_failed_registration_retains_old_binding() {
        let mut registry = ShortcutRegistry::seeded();
        let old = registry.standard_chord().clone();
        let mut registrar = FakeRegistrar { fail: true };
        let err = registry
            .register(
                chord(ShortcutActionId::CaptureSelection, "k"),
                &mut registrar,
                CaptureAlternatives {
                    chord: true,
                    menu: true,
                    manual: true,
                },
            )
            .unwrap_err();
        assert_eq!(err, RegisterError::NativeRejected);
        assert_eq!(registry.standard_chord(), &old);
        assert!(registry.standard_chord().enabled);
    }

    #[test]
    fn shortcuts_cannot_disable_chord_menu_and_manual_together() {
        assert!(!capture_alternatives_ok(CaptureAlternatives {
            chord: false,
            menu: false,
            manual: false,
        }));
        let mut registry = ShortcutRegistry::seeded();
        let mut registrar = FakeRegistrar { fail: false };
        let mut disabled = registry.standard_chord().clone();
        disabled.enabled = false;
        assert_eq!(
            registry
                .register(
                    disabled,
                    &mut registrar,
                    CaptureAlternatives {
                        chord: false,
                        menu: false,
                        manual: false,
                    },
                )
                .unwrap_err(),
            RegisterError::AlternativesRequired
        );
        assert!(capture_alternatives_ok(CaptureAlternatives {
            chord: false,
            menu: true,
            manual: false,
        }));
    }

    #[test]
    fn shortcuts_recorder_does_not_swallow_ime_or_voiceover() {
        assert_eq!(
            recorder_swallows(RecorderEvent {
                is_composing: true,
                is_repeat: false,
                is_voiceover_reserved: false,
                has_modifier: true,
                is_character: false,
                is_global: true,
            }),
            Err(RecorderIgnore::ImeComposing)
        );
        assert_eq!(
            recorder_swallows(RecorderEvent {
                is_composing: false,
                is_repeat: false,
                is_voiceover_reserved: true,
                has_modifier: true,
                is_character: false,
                is_global: true,
            }),
            Err(RecorderIgnore::VoiceOverReserved)
        );
        assert_eq!(
            recorder_swallows(RecorderEvent {
                is_composing: false,
                is_repeat: false,
                is_voiceover_reserved: false,
                has_modifier: true,
                is_character: false,
                is_global: true,
            }),
            Ok(true)
        );
        let mut binding = chord(ShortcutActionId::QueueCopy, "c");
        skip_test_marks_untested(&mut binding);
        assert_eq!(binding.tested, Some(TestedState::Skipped));
    }

    #[test]
    fn shortcuts_seed_docs_05_defaults_and_upgrade_factory_placeholders() {
        let registry = ShortcutRegistry::seeded();
        assert!(registry.get(ShortcutActionId::AppTogglePanel).enabled);
        assert_eq!(
            registry
                .get(ShortcutActionId::AppTogglePanel)
                .logical_key
                .as_deref(),
            Some(" ")
        );
        assert_eq!(
            registry.get(ShortcutActionId::AppTogglePanel).modifiers,
            vec![Modifier::Option]
        );
        assert_eq!(
            shortcut_scope(ShortcutActionId::AppTogglePanel),
            ShortcutScope::Global
        );
        assert_eq!(
            shortcut_scope(ShortcutActionId::CaptureSelection),
            ShortcutScope::Global
        );
        assert_eq!(
            shortcut_scope(ShortcutActionId::QueueCopy),
            ShortcutScope::AppLocal
        );
        assert_eq!(
            registry
                .get(ShortcutActionId::QueueCopy)
                .logical_key
                .as_deref(),
            Some("c")
        );
        assert_eq!(
            registry
                .get(ShortcutActionId::QueueCopyWithProfile)
                .modifiers,
            vec![Modifier::Command, Modifier::Shift]
        );
        assert_eq!(
            registry
                .get(ShortcutActionId::WindowSettings)
                .logical_key
                .as_deref(),
            Some(",")
        );
        assert!(is_default_binding(registry.standard_chord()));
        let placeholder = ShortcutBinding {
            action: ShortcutActionId::QueueSearch,
            trigger: TriggerKind::Disabled,
            modifiers: Vec::new(),
            key_mode: None,
            physical_code: None,
            logical_key: None,
            modifier_side: None,
            gap_ms: None,
            max_hold_ms: None,
            tap_count: None,
            enabled: false,
            schema_version: SCHEMA_VERSION,
            revision: 1,
            tested: Some(TestedState::Untested),
        };
        let json = serde_json::to_string(&placeholder).expect("json");
        let restored = ShortcutRegistry::from_rows(&[("queue.search".into(), json, false, 1)])
            .expect("upgrade");
        assert!(restored.get(ShortcutActionId::QueueSearch).enabled);
        assert_eq!(
            restored
                .get(ShortcutActionId::QueueSearch)
                .logical_key
                .as_deref(),
            Some("f")
        );
        let mut custom = registry;
        let mut registrar = FakeRegistrar { fail: false };
        custom
            .register(
                chord(ShortcutActionId::QueueSearch, "k"),
                &mut registrar,
                CaptureAlternatives {
                    chord: true,
                    menu: true,
                    manual: true,
                },
            )
            .expect("custom");
        assert!(!is_default_binding(
            custom.get(ShortcutActionId::QueueSearch)
        ));
        custom
            .restore_default(
                ShortcutActionId::QueueSearch,
                &mut registrar,
                CaptureAlternatives {
                    chord: true,
                    menu: true,
                    manual: true,
                },
            )
            .expect("restore");
        assert!(is_default_binding(
            custom.get(ShortcutActionId::QueueSearch)
        ));
    }

    #[test]
    fn recorded_double_tap_fills_default_timing_and_rejects_empty_modifiers() {
        let mut binding = default_shortcut_binding(ShortcutActionId::QueueSearch);
        binding.trigger = TriggerKind::ModifierDoubleTap;
        binding.modifiers = vec![Modifier::Option];
        binding.logical_key = Some("o".into());
        apply_recorded_double_tap_timing(&mut binding);
        assert_eq!(binding.logical_key, None);
        assert_eq!(binding.gap_ms, Some(DEFAULT_GAP_MS));
        assert_eq!(binding.max_hold_ms, Some(DEFAULT_MAX_HOLD_MS));
        assert_eq!(binding.modifier_side, Some(ModifierSide::Either));
        assert_eq!(binding.tap_count, Some(2));
        assert!(recorded_double_tap_allowed(&binding));
        binding.modifiers.clear();
        assert!(!recorded_double_tap_allowed(&binding));
        let mut triple = default_shortcut_binding(ShortcutActionId::QueueComplete);
        triple.trigger = TriggerKind::ModifierDoubleTap;
        triple.modifiers = vec![Modifier::Option];
        triple.logical_key = None;
        triple.tap_count = Some(3);
        apply_recorded_double_tap_timing(&mut triple);
        assert_eq!(triple.tap_count, Some(3));
        assert!(recorded_double_tap_allowed(&triple));
        assert_ne!(effective_tap_count(&binding), effective_tap_count(&triple));
        let mut missing = triple.clone();
        missing.tap_count = None;
        apply_recorded_double_tap_timing(&mut missing);
        assert_eq!(effective_tap_count(&missing), Some(2));
        let mut accelerator = default_shortcut_binding(ShortcutActionId::CaptureSelection);
        accelerator.trigger = TriggerKind::Accelerator;
        apply_recorded_double_tap_timing(&mut accelerator);
        assert_eq!(accelerator.gap_ms, None);
        assert_eq!(accelerator.max_hold_ms, None);
        assert_eq!(accelerator.modifier_side, None);
        assert_eq!(accelerator.tap_count, None);
    }

    #[test]
    fn live_shift_tap_count_follows_capture_selection_binding() {
        let double = default_shortcut_binding(ShortcutActionId::CaptureSelection);
        assert_eq!(live_shift_tap_count(&double), Some(2));
        let mut triple = double.clone();
        triple.tap_count = Some(3);
        assert_eq!(live_shift_tap_count(&triple), Some(3));
        let mut option = triple.clone();
        option.modifiers = vec![Modifier::Option];
        assert_eq!(live_shift_tap_count(&option), None);
        let mut accel = default_shortcut_binding(ShortcutActionId::CaptureSelection);
        accel.trigger = TriggerKind::Accelerator;
        accel.logical_key = Some("k".into());
        accel.tap_count = None;
        apply_recorded_double_tap_timing(&mut accel);
        assert_eq!(live_shift_tap_count(&accel), None);
        let search = default_shortcut_binding(ShortcutActionId::QueueSearch);
        assert_eq!(live_shift_tap_count(&search), None);
    }
}
