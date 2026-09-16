//! AX-first provider against fixtures (story 3.6, CAP-005/006/007/009).
//!
//! Classification happens before any content query. Exclusion is earlier still.

use std::cell::Cell;
use std::collections::HashSet;
use std::fmt;

pub const AX_MAX_SELECTION_BYTES: usize = 1 << 20;
const AX_CHAIN_LIMIT: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxRole {
    Application,
    Window,
    Group,
    ScrollArea,
    TextField,
    TextArea,
    StaticText,
    SecureTextField,
    WebArea,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxSubrole {
    SecureTextField,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtectionClass {
    Protected,
    AllowedText,
    NeutralContainer,
    UnknownContentBearing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FakeSelection {
    Missing,
    Empty,
    Text(String),
    InvalidUtf8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxOutcome {
    Captured { len: usize },
    NoSelection,
    ProtectedContent,
    ProtectionUnknown,
    AppExcluded,
    AccessibilityDenied,
    FocusedElementMissing,
    SelectionTooLarge,
    InvalidTextEncoding,
}

pub struct CapturedText {
    pub text: String,
}

impl fmt::Debug for CapturedText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CapturedText")
            .field("len", &self.text.len())
            .finish_non_exhaustive()
    }
}

pub struct FakeAxNode {
    pub role: AxRole,
    pub subrole: Option<AxSubrole>,
    pub selection: FakeSelection,
    pub parent: Option<usize>,
    queries: Cell<u32>,
}

impl FakeAxNode {
    pub fn new(
        role: AxRole,
        subrole: Option<AxSubrole>,
        selection: FakeSelection,
        parent: Option<usize>,
    ) -> Self {
        Self {
            role,
            subrole,
            selection,
            parent,
            queries: Cell::new(0),
        }
    }

    pub fn query_count(&self) -> u32 {
        self.queries.get()
    }

    fn query_selection(&self) -> FakeSelection {
        self.queries.set(self.queries.get() + 1);
        self.selection.clone()
    }
}

pub struct FakeAxTree {
    pub nodes: Vec<FakeAxNode>,
    pub focused: Option<usize>,
    pub excluded: bool,
    pub accessibility_granted: bool,
}

impl FakeAxTree {
    pub fn total_queries(&self) -> u32 {
        self.nodes.iter().map(FakeAxNode::query_count).sum()
    }
}

pub fn classify(role: AxRole, subrole: Option<AxSubrole>) -> ProtectionClass {
    if matches!(role, AxRole::SecureTextField) || subrole == Some(AxSubrole::SecureTextField) {
        return ProtectionClass::Protected;
    }
    match role {
        AxRole::Application | AxRole::Window | AxRole::Group | AxRole::ScrollArea => {
            ProtectionClass::NeutralContainer
        }
        AxRole::TextField | AxRole::TextArea | AxRole::StaticText => ProtectionClass::AllowedText,
        AxRole::WebArea | AxRole::Unknown => ProtectionClass::UnknownContentBearing,
        AxRole::SecureTextField => ProtectionClass::Protected,
    }
}

pub fn capture(tree: &FakeAxTree) -> (AxOutcome, Option<CapturedText>) {
    if !tree.accessibility_granted {
        return (AxOutcome::AccessibilityDenied, None);
    }
    if tree.excluded {
        return (AxOutcome::AppExcluded, None);
    }
    let Some(start) = tree.focused else {
        return (AxOutcome::FocusedElementMissing, None);
    };
    let mut seen = HashSet::new();
    let mut idx = Some(start);
    let mut steps = 0;
    while let Some(i) = idx {
        if steps >= AX_CHAIN_LIMIT || !seen.insert(i) {
            break;
        }
        steps += 1;
        let Some(node) = tree.nodes.get(i) else {
            break;
        };
        match classify(node.role, node.subrole) {
            ProtectionClass::Protected => return (AxOutcome::ProtectedContent, None),
            ProtectionClass::UnknownContentBearing => return (AxOutcome::ProtectionUnknown, None),
            ProtectionClass::NeutralContainer => {
                idx = node.parent;
            }
            ProtectionClass::AllowedText => match node.query_selection() {
                FakeSelection::Missing | FakeSelection::Empty => idx = node.parent,
                FakeSelection::InvalidUtf8 => return (AxOutcome::InvalidTextEncoding, None),
                FakeSelection::Text(text) => {
                    if text.is_empty() {
                        idx = node.parent;
                    } else if text.len() > AX_MAX_SELECTION_BYTES {
                        return (AxOutcome::SelectionTooLarge, None);
                    } else {
                        let len = text.len();
                        return (AxOutcome::Captured { len }, Some(CapturedText { text }));
                    }
                }
            },
        }
    }
    (AxOutcome::NoSelection, None)
}

pub fn store_capture(store: &mut Vec<String>, outcome: AxOutcome, text: Option<CapturedText>) {
    if let (AxOutcome::Captured { .. }, Some(captured)) = (outcome, text) {
        store.push(captured.text);
    }
}

#[cfg(test)]
mod ax_tests {
    use super::*;

    fn secret_field() -> FakeAxNode {
        FakeAxNode::new(
            AxRole::TextField,
            Some(AxSubrole::SecureTextField),
            FakeSelection::Text("hunter2-secret".into()),
            None,
        )
    }

    #[test]
    fn ax_secure_and_unknown_fail_closed_with_zero_content() {
        let secure = FakeAxTree {
            nodes: vec![secret_field()],
            focused: Some(0),
            excluded: false,
            accessibility_granted: true,
        };
        let (outcome, text) = capture(&secure);
        assert_eq!(outcome, AxOutcome::ProtectedContent);
        assert!(text.is_none());
        assert_eq!(secure.total_queries(), 0);
        let rendered = format!("{outcome:?}{text:?}");
        assert!(!rendered.contains("hunter2"));
        let mut store = Vec::new();
        store_capture(&mut store, outcome, text);
        assert!(store.is_empty());

        let unknown = FakeAxTree {
            nodes: vec![FakeAxNode::new(
                AxRole::WebArea,
                None,
                FakeSelection::Text("do-not-read".into()),
                None,
            )],
            focused: Some(0),
            excluded: false,
            accessibility_granted: true,
        };
        let (outcome, text) = capture(&unknown);
        assert_eq!(outcome, AxOutcome::ProtectionUnknown);
        assert!(text.is_none());
        assert_eq!(unknown.total_queries(), 0);
        assert!(!format!("{outcome:?}").contains("do-not-read"));
    }

    #[test]
    fn ax_exclusion_runs_before_content_query() {
        let tree = FakeAxTree {
            nodes: vec![FakeAxNode::new(
                AxRole::TextArea,
                None,
                FakeSelection::Text("excluded-body".into()),
                None,
            )],
            focused: Some(0),
            excluded: true,
            accessibility_granted: true,
        };
        let (outcome, text) = capture(&tree);
        assert_eq!(outcome, AxOutcome::AppExcluded);
        assert!(text.is_none());
        assert_eq!(tree.total_queries(), 0);
    }

    #[test]
    fn ax_whitespace_preserved_exactly() {
        let raw = "  hello\r\n\t ";
        let tree = FakeAxTree {
            nodes: vec![FakeAxNode::new(
                AxRole::TextArea,
                None,
                FakeSelection::Text(raw.into()),
                None,
            )],
            focused: Some(0),
            excluded: false,
            accessibility_granted: true,
        };
        let (outcome, text) = capture(&tree);
        assert_eq!(outcome, AxOutcome::Captured { len: raw.len() });
        assert_eq!(text.expect("text").text, raw);
    }

    #[test]
    fn ax_empty_and_zero_width_are_no_selection_without_trim() {
        let empty = FakeAxTree {
            nodes: vec![FakeAxNode::new(
                AxRole::TextField,
                None,
                FakeSelection::Empty,
                None,
            )],
            focused: Some(0),
            excluded: false,
            accessibility_granted: true,
        };
        let (outcome, text) = capture(&empty);
        assert_eq!(outcome, AxOutcome::NoSelection);
        assert!(text.is_none());

        let spaces = FakeAxTree {
            nodes: vec![FakeAxNode::new(
                AxRole::TextField,
                None,
                FakeSelection::Text("   ".into()),
                None,
            )],
            focused: Some(0),
            excluded: false,
            accessibility_granted: true,
        };
        let (outcome, text) = capture(&spaces);
        assert_eq!(outcome, AxOutcome::Captured { len: 3 });
        assert_eq!(text.expect("spaces").text, "   ");
    }

    #[test]
    fn ax_neutral_then_allowed_walks_without_querying_containers() {
        let tree = FakeAxTree {
            nodes: vec![
                FakeAxNode::new(AxRole::Group, None, FakeSelection::Missing, Some(1)),
                FakeAxNode::new(
                    AxRole::StaticText,
                    None,
                    FakeSelection::Text("leaf".into()),
                    None,
                ),
            ],
            focused: Some(0),
            excluded: false,
            accessibility_granted: true,
        };
        let (outcome, text) = capture(&tree);
        assert_eq!(outcome, AxOutcome::Captured { len: 4 });
        assert_eq!(text.expect("leaf").text, "leaf");
        assert_eq!(tree.nodes[0].query_count(), 0);
        assert_eq!(tree.nodes[1].query_count(), 1);
    }

    #[test]
    fn ax_protected_debug_and_store_never_hold_secret() {
        let tree = FakeAxTree {
            nodes: vec![secret_field()],
            focused: Some(0),
            excluded: false,
            accessibility_granted: true,
        };
        let (outcome, text) = capture(&tree);
        let shown = format!("{outcome:?}{text:?}");
        let mut store = Vec::new();
        store_capture(&mut store, outcome, text);
        let blob = format!("{shown}{store:?}");
        assert!(!blob.contains("hunter2"));
        assert!(!blob.contains("secret"));
    }
}
