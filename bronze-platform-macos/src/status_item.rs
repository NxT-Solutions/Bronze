//! Accessible status menu (story 5.1, CAP-003, WIN-004, A11Y-001, I18N-001).

use crate::abi::BronzeIngressSnapshot;
use crate::bridge::{NativeError, NativeRuntime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatusAction {
    Capture,
    NewNote,
    Show,
    Settings,
    Quit,
}

pub const STATUS_MENU_KEYS: &[(StatusAction, &str)] = &[
    (StatusAction::Capture, "menu.status.capture"),
    (StatusAction::NewNote, "menu.status.newNote"),
    (StatusAction::Show, "menu.status.show"),
    (StatusAction::Settings, "menu.status.settings"),
    (StatusAction::Quit, "menu.status.quit"),
];

pub trait StringCatalog {
    fn accessible_name(&self, key: &str) -> Option<String>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusMenuItem {
    pub action: StatusAction,
    pub catalog_key: &'static str,
    pub accessible_name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatusMenuError {
    MissingAccessibleName,
}

pub fn build_status_menu(
    catalog: &dyn StringCatalog,
) -> Result<Vec<StatusMenuItem>, StatusMenuError> {
    let mut items = Vec::with_capacity(STATUS_MENU_KEYS.len());
    for (action, key) in STATUS_MENU_KEYS {
        let accessible_name = catalog
            .accessible_name(key)
            .filter(|s| !s.is_empty())
            .ok_or(StatusMenuError::MissingAccessibleName)?;
        items.push(StatusMenuItem {
            action: *action,
            catalog_key: key,
            accessible_name,
        });
    }
    Ok(items)
}

impl NativeRuntime {
    pub fn capture_from_status_menu(&self) -> Result<BronzeIngressSnapshot, NativeError> {
        self.ingress_load()
    }

    pub fn status_menu_available_without_event_tap(&self) -> Result<bool, NativeError> {
        self.event_tap_set_enabled(false)?;
        let health = self.event_tap_health()?;
        let _ = self.capture_from_status_menu()?;
        Ok(health != crate::bridge::EventTapHealth::Listening)
    }
}

#[cfg(test)]
mod status_item_tests {
    use super::*;
    use crate::bridge::{EventTapHealth, NativeRuntime};
    use crate::lock_native_runtime as lock_runtime;
    use std::collections::BTreeMap;

    struct MapCatalog(BTreeMap<&'static str, &'static str>);

    impl StringCatalog for MapCatalog {
        fn accessible_name(&self, key: &str) -> Option<String> {
            self.0.get(key).map(|s| (*s).to_string())
        }
    }

    fn full_catalog() -> MapCatalog {
        MapCatalog(
            STATUS_MENU_KEYS
                .iter()
                .map(|(action, key)| {
                    let name = match action {
                        StatusAction::Capture => "Capture",
                        StatusAction::NewNote => "New Note",
                        StatusAction::Show => "Show",
                        StatusAction::Settings => "Settings",
                        StatusAction::Quit => "Quit",
                    };
                    (*key, name)
                })
                .collect(),
        )
    }

    #[test]
    fn status_item_menu_requires_localized_accessible_names() {
        let items = build_status_menu(&full_catalog()).expect("menu");
        assert_eq!(items.len(), 5);
        assert!(items.iter().all(|i| !i.accessible_name.is_empty()));
        assert_eq!(items[0].action, StatusAction::Capture);
        assert_eq!(items[0].catalog_key, "menu.status.capture");
        let mut incomplete = full_catalog();
        incomplete.0.remove("menu.status.show");
        assert_eq!(
            build_status_menu(&incomplete).unwrap_err(),
            StatusMenuError::MissingAccessibleName
        );
    }

    #[test]
    fn status_item_capture_uses_last_external_target_snapshot() {
        let _guard = lock_runtime();
        let runtime = NativeRuntime::start().expect("start");
        let snap = BronzeIngressSnapshot {
            target_pid: 4242,
            bundle_token: 7,
            activation_generation: 3,
            destination_uuid: [9; 16],
            accept_capture_generation: 1,
            policy_revision: 2,
            settings_revision: 4,
            context_generation: 5,
            route: 1,
            monotonic_time_ns: 99,
        };
        runtime.ingress_publish(snap).expect("publish");
        let loaded = runtime.capture_from_status_menu().expect("capture");
        assert_eq!(loaded.target_pid, 4242);
        assert_eq!(loaded.bundle_token, 7);
        assert_eq!(loaded.activation_generation, 3);
        runtime.shutdown().expect("shutdown");
    }

    #[test]
    fn status_item_menu_works_when_event_tap_is_off() {
        let _guard = lock_runtime();
        let runtime = NativeRuntime::start().expect("start");
        let _ = runtime.event_tap_start();
        runtime.event_tap_set_enabled(false).expect("disable");
        assert!(runtime
            .status_menu_available_without_event_tap()
            .expect("menu"));
        assert_ne!(
            runtime.event_tap_health().expect("health"),
            EventTapHealth::Listening
        );
        let items = build_status_menu(&full_catalog()).expect("names");
        assert_eq!(items.len(), 5);
        runtime.shutdown().expect("shutdown");
    }
}
