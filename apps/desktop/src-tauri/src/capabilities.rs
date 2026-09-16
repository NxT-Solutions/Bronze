//! Per-window command allow-list (story 6.3, QUE-001/008, WIN-005, SEC-002).

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowKind {
    Quick,
    Library,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowCommand {
    Import,
    Export,
    Backup,
    Paginate,
    Archive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LibraryViewState {
    Empty,
    Loading,
    Ready,
    ReadOnly,
}

pub fn window_allows(window: WindowKind, command: WindowCommand) -> bool {
    match (window, command) {
        (
            WindowKind::Quick,
            WindowCommand::Import | WindowCommand::Export | WindowCommand::Backup,
        ) => false,
        (WindowKind::Quick, _) => false,
        (WindowKind::Library, WindowCommand::Paginate | WindowCommand::Archive) => true,
        (WindowKind::Library, _) => false,
    }
}

#[cfg(test)]
mod capabilities_tests {
    use super::*;
    use serde_json::Value;
    use std::fs;

    #[test]
    fn capabilities_quick_denies_import_export_backup_library_paginates() {
        assert!(!window_allows(WindowKind::Quick, WindowCommand::Import));
        assert!(!window_allows(WindowKind::Quick, WindowCommand::Export));
        assert!(!window_allows(WindowKind::Quick, WindowCommand::Backup));
        assert!(window_allows(WindowKind::Library, WindowCommand::Paginate));
        assert!(window_allows(WindowKind::Library, WindowCommand::Archive));
        assert!(!window_allows(WindowKind::Library, WindowCommand::Import));
        let states = [
            LibraryViewState::Empty,
            LibraryViewState::Loading,
            LibraryViewState::Ready,
            LibraryViewState::ReadOnly,
        ];
        assert_eq!(states.len(), 4);

        let conf: Value = serde_json::from_str(
            &fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json"))
                .expect("conf"),
        )
        .expect("json");
        let labels: Vec<&str> = conf["app"]["windows"]
            .as_array()
            .expect("windows")
            .iter()
            .map(|w| w["label"].as_str().expect("label"))
            .collect();
        assert!(labels.contains(&"quick"));
        assert!(labels.contains(&"library"));
    }
}
