//! Shortcut registry and recorder policy (story 7.2, SET-002, CAP-001, CAP-002).

use crate::schema::{ShortcutActionId, ShortcutBinding, TestedState, TriggerKind, SCHEMA_VERSION};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegisterError {
    NativeRejected,
    Duplicate,
    AlternativesRequired,
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
}

fn default_binding(action: ShortcutActionId) -> ShortcutBinding {
    if action == ShortcutActionId::CaptureSelection {
        return crate::schema::default_standard_chord();
    }
    ShortcutBinding {
        action,
        trigger: TriggerKind::Disabled,
        modifiers: Vec::new(),
        key_mode: None,
        physical_code: None,
        logical_key: None,
        modifier_side: None,
        gap_ms: None,
        max_hold_ms: None,
        enabled: false,
        schema_version: SCHEMA_VERSION,
        revision: 1,
        tested: Some(TestedState::Untested),
    }
}

fn same_chord(left: &ShortcutBinding, right: &ShortcutBinding) -> bool {
    left.trigger == right.trigger
        && left.modifiers == right.modifiers
        && left.logical_key == right.logical_key
        && left.physical_code == right.physical_code
        && left.trigger != TriggerKind::Disabled
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
}
