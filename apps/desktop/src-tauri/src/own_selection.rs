//! Fixed-script read of the current Bronze WebView highlight.

use bronze_platform_macos::LIVE_AX_MAX_BYTES;
use std::fmt;

pub const OWN_SELECTION_WINDOW_LABELS: &[&str] = &["quick", "settings", "library", "help"];
pub const OWN_SELECTION_SOURCE_NAME: &str = "Bronze";
pub const OWN_SELECTION_WAIT: std::time::Duration = std::time::Duration::from_millis(400);

pub const OWN_SELECTION_JS: &str = r#"(function(){
  try {
    var el = document.activeElement;
    var tag = el ? String(el.tagName || "").toUpperCase() : "";
    var type = el ? String(el.type || "").toLowerCase() : "";
    if (type === "password") {
      return "";
    }
    if (tag === "TEXTAREA" || tag === "INPUT") {
      var start = el.selectionStart;
      var end = el.selectionEnd;
      if (typeof start === "number" && typeof end === "number" && end > start) {
        return String(el.value || "").slice(start, end);
      }
    }
    var sel = window.getSelection();
    return sel ? String(sel.toString()) : "";
  } catch (err) {
    return "";
  }
})()"#;

#[derive(Clone, Eq, PartialEq)]
pub enum OwnSelection {
    Text(String),
    TooLarge,
}

impl fmt::Debug for OwnSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(text) => f.debug_tuple("Text").field(&text.len()).finish(),
            Self::TooLarge => write!(f, "TooLarge"),
        }
    }
}

pub fn decode_own_selection_json(json: &str) -> Option<OwnSelection> {
    let text = serde_json::from_str::<String>(json).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.len() > LIVE_AX_MAX_BYTES {
        return Some(OwnSelection::TooLarge);
    }
    Some(OwnSelection::Text(trimmed.to_string()))
}

#[cfg(target_os = "macos")]
pub fn read_own_webview_selection(app: &tauri::AppHandle) -> Option<OwnSelection> {
    use tauri::Manager;
    let mut focused = Vec::new();
    let mut rest = Vec::new();
    for label in OWN_SELECTION_WINDOW_LABELS {
        let Some(window) = app.get_webview_window(label) else {
            continue;
        };
        if window.is_focused().ok() == Some(true) {
            focused.push(window);
        } else {
            rest.push(window);
        }
    }
    focused.append(&mut rest);
    if focused.is_empty() {
        return None;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    let mut expected = 0usize;
    let slot_count = focused.len();
    for (index, window) in focused.into_iter().enumerate() {
        let tx = tx.clone();
        if window
            .eval_with_callback(OWN_SELECTION_JS, move |json| {
                let _ = tx.send((index, decode_own_selection_json(&json)));
            })
            .is_ok()
        {
            expected += 1;
        }
    }
    drop(tx);
    if expected == 0 {
        return None;
    }
    let deadline = std::time::Instant::now() + OWN_SELECTION_WAIT;
    let mut slots: Vec<Option<OwnSelection>> = vec![None; slot_count];
    let mut got = 0usize;
    while got < expected {
        let remain = deadline.saturating_duration_since(std::time::Instant::now());
        if remain.is_zero() {
            break;
        }
        match rx.recv_timeout(remain) {
            Ok((index, value)) => {
                if let Some(slot) = slots.get_mut(index) {
                    *slot = value;
                }
                got += 1;
            }
            Err(_) => break,
        }
    }
    slots.into_iter().flatten().next()
}

#[cfg(test)]
mod own_selection_tests {
    use super::*;

    #[test]
    fn decode_own_selection_json_accepts_highlight_and_rejects_empty() {
        assert_eq!(
            decode_own_selection_json("\"inside bronze\""),
            Some(OwnSelection::Text("inside bronze".into()))
        );
        assert_eq!(
            decode_own_selection_json("\"  padded  \""),
            Some(OwnSelection::Text("padded".into()))
        );
        assert_eq!(decode_own_selection_json("\"\""), None);
        assert_eq!(decode_own_selection_json("\"   \""), None);
        assert_eq!(decode_own_selection_json(""), None);
        assert_eq!(decode_own_selection_json("null"), None);
        assert_eq!(
            decode_own_selection_json(&serde_json::to_string(&"line\nbreak").expect("json")),
            Some(OwnSelection::Text("line\nbreak".into()))
        );
        let too_large = "x".repeat(LIVE_AX_MAX_BYTES + 1);
        assert_eq!(
            decode_own_selection_json(&serde_json::to_string(&too_large).expect("json")),
            Some(OwnSelection::TooLarge)
        );
        assert!(!format!("{:?}", OwnSelection::Text("secret-body".into())).contains("secret"));
    }

    #[test]
    fn own_selection_script_covers_fields_and_stays_local() {
        assert!(OWN_SELECTION_JS.contains("getSelection"));
        assert!(OWN_SELECTION_JS.contains("selectionStart"));
        assert!(OWN_SELECTION_JS.contains("selectionEnd"));
        assert!(OWN_SELECTION_JS.contains("TEXTAREA"));
        assert!(OWN_SELECTION_JS.contains("password"));
        assert!(OWN_SELECTION_JS.contains("slice"));
        assert!(!OWN_SELECTION_JS.contains("invoke"));
        assert!(!OWN_SELECTION_JS.contains("fetch("));
        assert!(!OWN_SELECTION_JS.contains("require("));
        assert!(!OWN_SELECTION_JS.contains("__TAURI__"));
        assert_eq!(
            OWN_SELECTION_WINDOW_LABELS,
            ["quick", "settings", "library", "help"]
        );
        assert_eq!(OWN_SELECTION_SOURCE_NAME, "Bronze");
        assert_eq!(LIVE_AX_MAX_BYTES, 1 << 20);
    }
}
