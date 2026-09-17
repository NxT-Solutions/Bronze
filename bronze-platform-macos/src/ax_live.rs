//! Focused AX selected-text read. Secure fields are never queried.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiveAxOutcome {
    Captured { len: usize },
    NoSelection,
    ProtectedContent,
    ProtectionUnknown,
    AccessibilityDenied,
    FocusedElementMissing,
    SelectionTooLarge,
    InvalidTextEncoding,
}

pub const LIVE_AX_MAX_BYTES: usize = 1 << 20;
const AX_CHAIN_LIMIT: usize = 16;
const AX_CHILD_BUDGET: usize = 80;
const AX_WINDOW_LIMIT: usize = 8;
const AX_CHILD_FANOUT: isize = 32;

pub fn use_system_focused_fallback(last_external: Option<i32>) -> bool {
    last_external.is_none()
}

pub fn reject_own_system_focus(system_pid: Option<i32>, own_pid: i32, name: Option<&str>) -> bool {
    match system_pid {
        None => true,
        Some(pid) if pid <= 0 || pid == own_pid => true,
        Some(_) => name.is_some_and(is_skipped_process_name),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxProtection {
    Protected,
    AllowedText,
    NeutralContainer,
    UnknownContentBearing,
}

pub fn classify_ax_role(role: &str, subrole: &str) -> AxProtection {
    if role == "AXSecureTextField" || subrole == "AXSecureTextField" {
        return AxProtection::Protected;
    }
    match role {
        "AXTextField" | "AXTextArea" | "AXStaticText" | "AXWebArea" | "AXText" | "AXComboBox"
        | "AXBrowser" | "AXDocument" => AxProtection::AllowedText,
        "AXApplication" | "AXWindow" | "AXGroup" | "AXScrollArea" | "AXLayoutArea"
        | "AXToolbar" | "AXMenuBar" | "AXMenu" | "AXSplitter" | "AXTabGroup" => {
            AxProtection::NeutralContainer
        }
        _ => AxProtection::UnknownContentBearing,
    }
}

pub fn is_skipped_process_name(name: &str) -> bool {
    name.eq_ignore_ascii_case("bronze-desktop") || name.eq_ignore_ascii_case("Bronze")
}

pub fn last_external_pid() -> Option<i32> {
    let pid = LAST_EXTERNAL_PID.load(std::sync::atomic::Ordering::SeqCst);
    (pid > 0).then_some(pid)
}

static LAST_EXTERNAL_PID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

#[cfg(target_os = "macos")]
mod sys {
    use std::ffi::{c_char, c_void};

    pub type CfTypeRef = *const c_void;
    pub type CfStringRef = *const c_void;
    pub type AxUiElementRef = *mut c_void;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct CfRange {
        pub location: isize,
        pub length: isize,
    }

    pub const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
    pub const AX_SUCCESS: i32 = 0;

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        pub fn CFRelease(cf: CfTypeRef);
        pub fn CFGetTypeID(cf: CfTypeRef) -> usize;
        pub fn CFStringGetTypeID() -> usize;
        pub fn CFStringCreateWithCString(
            alloc: *const c_void,
            c_str: *const c_char,
            encoding: u32,
        ) -> CfStringRef;
        pub fn CFStringGetLength(the_string: CfStringRef) -> isize;
        pub fn CFStringGetMaximumSizeForEncoding(length: isize, encoding: u32) -> isize;
        pub fn CFStringGetCString(
            the_string: CfStringRef,
            buffer: *mut c_char,
            buffer_size: isize,
            encoding: u32,
        ) -> u8;
        pub fn CFArrayGetTypeID() -> usize;
        pub fn CFArrayGetCount(the_array: CfTypeRef) -> isize;
        pub fn CFArrayGetValueAtIndex(the_array: CfTypeRef, idx: isize) -> CfTypeRef;
        pub fn CFAttributedStringGetTypeID() -> usize;
        pub fn CFAttributedStringGetLength(a_str: CfTypeRef) -> isize;
        pub fn CFAttributedStringGetAttributes(
            a_str: CfTypeRef,
            loc: isize,
            effective_range: *mut CfRange,
        ) -> CfTypeRef;
        pub fn CFAttributedStringGetString(a_str: CfTypeRef) -> CfStringRef;
        pub fn CFStringCreateWithSubstring(
            alloc: *const c_void,
            the_string: CfStringRef,
            range: CfRange,
        ) -> CfStringRef;
        pub fn CFDictionaryGetTypeID() -> usize;
        pub fn CFDictionaryGetValue(the_dict: CfTypeRef, key: CfTypeRef) -> CfTypeRef;
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        pub fn AXIsProcessTrusted() -> u8;
        pub fn AXUIElementCreateSystemWide() -> AxUiElementRef;
        pub fn AXUIElementCopyAttributeValue(
            element: AxUiElementRef,
            attribute: CfStringRef,
            value: *mut CfTypeRef,
        ) -> i32;
        pub fn AXUIElementGetPid(element: AxUiElementRef, pid: *mut i32) -> i32;
        pub fn AXUIElementCreateApplication(pid: i32) -> AxUiElementRef;
        pub fn AXUIElementCopyParameterizedAttributeValue(
            element: AxUiElementRef,
            attribute: CfStringRef,
            parameter: CfTypeRef,
            value: *mut CfTypeRef,
        ) -> i32;
        pub fn AXUIElementSetMessagingTimeout(
            element: AxUiElementRef,
            timeout_in_seconds: f32,
        ) -> i32;
    }

    extern "C" {
        pub fn proc_name(pid: i32, buffer: *mut c_void, buffersize: u32) -> i32;
    }

    pub fn cf_string(name: &str) -> Option<CfStringRef> {
        let c = std::ffi::CString::new(name).ok()?;
        let raw = unsafe {
            CFStringCreateWithCString(std::ptr::null(), c.as_ptr(), K_CF_STRING_ENCODING_UTF8)
        };
        if raw.is_null() {
            None
        } else {
            Some(raw)
        }
    }

    pub fn cf_string_to_owned(value: CfTypeRef) -> Result<String, ()> {
        unsafe {
            if value.is_null() || CFGetTypeID(value) != CFStringGetTypeID() {
                return Err(());
            }
            let len = CFStringGetLength(value);
            let cap = CFStringGetMaximumSizeForEncoding(len, K_CF_STRING_ENCODING_UTF8) + 1;
            if cap <= 0 {
                return Ok(String::new());
            }
            let mut buf = vec![0u8; cap as usize];
            if CFStringGetCString(
                value,
                buf.as_mut_ptr().cast::<c_char>(),
                cap,
                K_CF_STRING_ENCODING_UTF8,
            ) == 0
            {
                return Err(());
            }
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            String::from_utf8(buf[..end].to_vec()).map_err(|_| ())
        }
    }

    pub fn pid_for_element(element: AxUiElementRef) -> Option<i32> {
        let mut pid: i32 = 0;
        let status = unsafe { AXUIElementGetPid(element, &mut pid) };
        if status != AX_SUCCESS || pid <= 0 {
            None
        } else {
            Some(pid)
        }
    }

    pub fn process_name(pid: i32) -> Option<String> {
        let mut buf = [0u8; 256];
        let n = unsafe { proc_name(pid, buf.as_mut_ptr().cast(), buf.len() as u32) };
        if n <= 0 {
            return None;
        }
        let raw = std::str::from_utf8(&buf[..n as usize]).ok()?.trim();
        if raw.is_empty() {
            None
        } else {
            Some(raw.chars().filter(|c| !c.is_control()).take(64).collect())
        }
    }

    pub fn app_name_for_element(element: AxUiElementRef) -> Option<String> {
        let pid = pid_for_element(element)?;
        let raw = process_name(pid)?;
        if is_own_pid(pid) || super::is_skipped_process_name(&raw) {
            return None;
        }
        Some(raw)
    }

    pub fn is_own_pid(pid: i32) -> bool {
        pid == std::process::id() as i32
    }

    pub fn remember_external_pid(pid: i32) {
        if pid <= 0 || is_own_pid(pid) {
            return;
        }
        if process_name(pid).is_some_and(|name| super::is_skipped_process_name(&name)) {
            return;
        }
        super::LAST_EXTERNAL_PID.store(pid, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn copy_param(
        element: AxUiElementRef,
        name: &str,
        parameter: CfTypeRef,
    ) -> Option<CfTypeRef> {
        let attr = cf_string(name)?;
        let mut out = std::ptr::null();
        let status = unsafe {
            AXUIElementCopyParameterizedAttributeValue(element, attr, parameter, &mut out)
        };
        unsafe { CFRelease(attr) };
        if status != AX_SUCCESS || out.is_null() {
            None
        } else {
            Some(out)
        }
    }

    fn bound_selection(text: String) -> Result<Option<String>, super::LiveAxOutcome> {
        if text.is_empty() {
            Ok(None)
        } else if text.len() > super::LIVE_AX_MAX_BYTES {
            Err(super::LiveAxOutcome::SelectionTooLarge)
        } else {
            Ok(Some(text))
        }
    }

    fn dict_string(dict: CfTypeRef, key_name: &str) -> Option<String> {
        let key = cf_string(key_name)?;
        let value = unsafe { CFDictionaryGetValue(dict, key) };
        unsafe { CFRelease(key) };
        if value.is_null() {
            None
        } else {
            cf_string_to_owned(value).ok()
        }
    }

    fn font_traits_from_attrs(attrs: CfTypeRef) -> (bool, bool) {
        if attrs.is_null() {
            return (false, false);
        }
        unsafe {
            if CFGetTypeID(attrs) != CFDictionaryGetTypeID() {
                return (false, false);
            }
        }
        let Some(key) = cf_string("AXFont") else {
            return (false, false);
        };
        let font = unsafe { CFDictionaryGetValue(attrs, key) };
        unsafe { CFRelease(key) };
        if font.is_null() {
            return (false, false);
        }
        unsafe {
            if CFGetTypeID(font) != CFDictionaryGetTypeID() {
                return (false, false);
            }
        }
        if let Some(name) = dict_string(font, "AXFontName") {
            return bronze_domain::font_name_traits(&name);
        }
        if let Some(name) = dict_string(font, "AXFontFamily") {
            return bronze_domain::font_name_traits(&name);
        }
        (false, false)
    }

    fn attributed_to_markdown(value: CfTypeRef) -> Result<String, super::LiveAxOutcome> {
        unsafe {
            let len = CFAttributedStringGetLength(value);
            if len <= 0 {
                return Ok(String::new());
            }
            let mut runs = Vec::new();
            let mut loc: isize = 0;
            let mut total = 0usize;
            while loc < len {
                let mut effective = CfRange {
                    location: 0,
                    length: 0,
                };
                let attrs = CFAttributedStringGetAttributes(value, loc, &mut effective);
                if effective.length <= 0 {
                    break;
                }
                let full = CFAttributedStringGetString(value);
                let sub = CFStringCreateWithSubstring(std::ptr::null(), full, effective);
                let text = if sub.is_null() {
                    String::new()
                } else {
                    let owned = cf_string_to_owned(sub).unwrap_or_default();
                    CFRelease(sub);
                    owned
                };
                total = total.saturating_add(text.len());
                if total > super::LIVE_AX_MAX_BYTES {
                    return Err(super::LiveAxOutcome::SelectionTooLarge);
                }
                let (bold, italic) = font_traits_from_attrs(attrs);
                if !text.is_empty() {
                    runs.push(bronze_domain::StyleRun { text, bold, italic });
                }
                let next = effective.location.saturating_add(effective.length);
                if next <= loc {
                    break;
                }
                loc = next;
            }
            Ok(bronze_domain::markdown_from_runs(&runs))
        }
    }

    pub fn take_text(value: CfTypeRef) -> Result<Option<String>, super::LiveAxOutcome> {
        unsafe {
            if value.is_null() {
                return Ok(None);
            }
            let type_id = CFGetTypeID(value);
            if type_id == CFAttributedStringGetTypeID() {
                let text = match attributed_to_markdown(value) {
                    Ok(text) => text,
                    Err(outcome) => {
                        CFRelease(value);
                        return Err(outcome);
                    }
                };
                CFRelease(value);
                return bound_selection(text);
            }
            if type_id != CFStringGetTypeID() {
                CFRelease(value);
                return Err(super::LiveAxOutcome::InvalidTextEncoding);
            }
        }
        let text = match cf_string_to_owned(value) {
            Ok(text) => text,
            Err(()) => {
                unsafe { CFRelease(value) };
                return Err(super::LiveAxOutcome::InvalidTextEncoding);
            }
        };
        unsafe { CFRelease(value) };
        bound_selection(text)
    }

    pub fn selected_text(element: AxUiElementRef) -> Result<Option<String>, super::LiveAxOutcome> {
        let range = copy_attr(element, "AXSelectedTextRange");
        if let Some(range) = range {
            if let Some(value) = copy_param(element, "AXAttributedStringForRange", range) {
                match take_text(value) {
                    Ok(Some(text)) => {
                        unsafe { CFRelease(range) };
                        return Ok(Some(text));
                    }
                    Ok(None) => {}
                    Err(outcome) => {
                        unsafe { CFRelease(range) };
                        return Err(outcome);
                    }
                }
            }
            if let Some(selected) = copy_attr(element, "AXSelectedText") {
                match take_text(selected) {
                    Ok(Some(text)) => {
                        unsafe { CFRelease(range) };
                        return Ok(Some(text));
                    }
                    Ok(None) => {}
                    Err(outcome) => {
                        unsafe { CFRelease(range) };
                        return Err(outcome);
                    }
                }
            }
            let parameterized = copy_param(element, "AXStringForRange", range);
            unsafe { CFRelease(range) };
            match parameterized {
                Some(value) => take_text(value),
                None => Ok(None),
            }
        } else if let Some(selected) = copy_attr(element, "AXSelectedText") {
            take_text(selected)
        } else {
            Ok(None)
        }
    }

    fn take_owned(owned: &mut Option<CfTypeRef>) {
        if let Some(value) = owned.take() {
            unsafe { CFRelease(value) };
        }
    }

    fn attr_string(element: AxUiElementRef, name: &str) -> String {
        let Some(raw) = copy_attr(element, name) else {
            return String::new();
        };
        let text = cf_string_to_owned(raw).unwrap_or_default();
        unsafe { CFRelease(raw) };
        text
    }

    fn array_len(value: CfTypeRef) -> Option<isize> {
        unsafe {
            if value.is_null() || CFGetTypeID(value) != CFArrayGetTypeID() {
                None
            } else {
                Some(CFArrayGetCount(value))
            }
        }
    }

    fn array_get(value: CfTypeRef, idx: isize) -> Option<CfTypeRef> {
        let n = array_len(value)?;
        if idx < 0 || idx >= n {
            return None;
        }
        let item = unsafe { CFArrayGetValueAtIndex(value, idx) };
        if item.is_null() {
            None
        } else {
            Some(item)
        }
    }

    fn enqueue_children(
        element: AxUiElementRef,
        arrays: &mut Vec<CfTypeRef>,
        queue: &mut Vec<AxUiElementRef>,
    ) {
        let Some(children) = copy_attr(element, "AXChildren") else {
            return;
        };
        if let Some(n) = array_len(children) {
            for idx in 0..n.min(super::AX_CHILD_FANOUT) {
                if let Some(child) = array_get(children, idx) {
                    queue.push(child.cast_mut());
                }
            }
        }
        arrays.push(children);
    }

    type WalkHit = (super::LiveAxOutcome, Option<String>, Option<String>);

    fn inspect_node(
        element: AxUiElementRef,
        source_app_name: Option<String>,
    ) -> Result<Option<WalkHit>, super::LiveAxOutcome> {
        let role = attr_string(element, "AXRole");
        let subrole = attr_string(element, "AXSubrole");
        match super::classify_ax_role(&role, &subrole) {
            super::AxProtection::Protected => Err(super::LiveAxOutcome::ProtectedContent),
            super::AxProtection::AllowedText | super::AxProtection::NeutralContainer => {
                match selected_text(element) {
                    Ok(Some(text)) => {
                        let len = text.len();
                        Ok(Some((
                            super::LiveAxOutcome::Captured { len },
                            Some(text),
                            source_app_name,
                        )))
                    }
                    Ok(None) => Ok(None),
                    Err(outcome) => Err(outcome),
                }
            }
            super::AxProtection::UnknownContentBearing => Ok(None),
        }
    }

    fn search_children(start: AxUiElementRef, source_app_name: Option<String>) -> Option<WalkHit> {
        let mut arrays = Vec::new();
        let mut queue = Vec::new();
        let mut seen = std::collections::HashSet::new();
        enqueue_children(start, &mut arrays, &mut queue);
        let mut budget = super::AX_CHILD_BUDGET;
        let mut found = None;
        while budget > 0 {
            budget -= 1;
            let Some(element) = queue.pop() else {
                break;
            };
            if !seen.insert(element as usize) {
                continue;
            }
            match inspect_node(element, source_app_name.clone()) {
                Ok(Some(hit)) => {
                    found = Some(hit);
                    break;
                }
                Err(outcome) => {
                    found = Some((outcome, None, None));
                    break;
                }
                Ok(None) => enqueue_children(element, &mut arrays, &mut queue),
            }
        }
        for array in arrays {
            unsafe { CFRelease(array) };
        }
        found
    }

    pub fn walk_selection(start: AxUiElementRef) -> WalkHit {
        let source_app_name = app_name_for_element(start);
        let mut current = start;
        let mut owned: Option<CfTypeRef> = None;
        for _ in 0..super::AX_CHAIN_LIMIT {
            match inspect_node(current, source_app_name.clone()) {
                Ok(Some(hit)) => {
                    take_owned(&mut owned);
                    return hit;
                }
                Err(outcome) => {
                    take_owned(&mut owned);
                    return (outcome, None, None);
                }
                Ok(None) => {
                    if let Some(hit) = search_children(current, source_app_name.clone()) {
                        take_owned(&mut owned);
                        return hit;
                    }
                }
            }
            let parent = copy_attr(current, "AXParent");
            take_owned(&mut owned);
            match parent {
                Some(next) => {
                    owned = Some(next);
                    current = next.cast_mut();
                }
                None => break,
            }
        }
        take_owned(&mut owned);
        (super::LiveAxOutcome::NoSelection, None, source_app_name)
    }

    fn prefer_terminal(outcome: super::LiveAxOutcome) -> bool {
        matches!(
            outcome,
            super::LiveAxOutcome::Captured { .. }
                | super::LiveAxOutcome::ProtectedContent
                | super::LiveAxOutcome::ProtectionUnknown
                | super::LiveAxOutcome::SelectionTooLarge
                | super::LiveAxOutcome::InvalidTextEncoding
                | super::LiveAxOutcome::AccessibilityDenied
        )
    }

    pub fn walk_application(app: AxUiElementRef) -> WalkHit {
        let mut fallback = (
            super::LiveAxOutcome::FocusedElementMissing,
            None,
            app_name_for_element(app),
        );
        if let Some(focused) = copy_attr(app, "AXFocusedUIElement") {
            let result = walk_selection(focused.cast_mut());
            unsafe { CFRelease(focused) };
            if prefer_terminal(result.0) {
                return result;
            }
            fallback = result;
        }
        if let Some(main) = copy_attr(app, "AXMainWindow") {
            let result = walk_selection(main.cast_mut());
            unsafe { CFRelease(main) };
            if prefer_terminal(result.0) {
                return result;
            }
            if fallback.0 == super::LiveAxOutcome::FocusedElementMissing {
                fallback = result;
            }
        }
        if let Some(windows) = copy_attr(app, "AXWindows") {
            if let Some(n) = array_len(windows) {
                for idx in 0..n.min(super::AX_WINDOW_LIMIT as isize) {
                    if let Some(window) = array_get(windows, idx) {
                        let result = walk_selection(window.cast_mut());
                        if prefer_terminal(result.0) {
                            unsafe { CFRelease(windows) };
                            return result;
                        }
                    }
                }
            }
            unsafe { CFRelease(windows) };
        }
        fallback
    }

    pub fn copy_attr(element: AxUiElementRef, name: &str) -> Option<CfTypeRef> {
        let attr = cf_string(name)?;
        let mut out = std::ptr::null();
        let status = unsafe { AXUIElementCopyAttributeValue(element, attr, &mut out) };
        unsafe { CFRelease(attr) };
        if status != AX_SUCCESS || out.is_null() {
            None
        } else {
            Some(out)
        }
    }

    pub(super) fn set_messaging_timeout(element: AxUiElementRef) {
        // Native AX deadline (docs/07 §9.2), not a late-result check.
        unsafe {
            AXUIElementSetMessagingTimeout(element, 1.0);
        }
    }
}

#[cfg(target_os = "macos")]
fn trusted_or_denied() -> Option<(LiveAxOutcome, Option<String>, Option<String>)> {
    if unsafe { sys::AXIsProcessTrusted() } == 0 {
        Some((LiveAxOutcome::AccessibilityDenied, None, None))
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
fn read_system_focused() -> (LiveAxOutcome, Option<String>, Option<String>) {
    use sys::{
        copy_attr, pid_for_element, process_name, walk_selection, AXUIElementCreateSystemWide,
        CFRelease,
    };
    if let Some(denied) = trusted_or_denied() {
        return denied;
    }
    let system = unsafe { AXUIElementCreateSystemWide() };
    if system.is_null() {
        return (LiveAxOutcome::FocusedElementMissing, None, None);
    }
    sys::set_messaging_timeout(system);
    let focused = copy_attr(system, "AXFocusedUIElement");
    unsafe { CFRelease(system.cast()) };
    let Some(focused) = focused else {
        return (LiveAxOutcome::FocusedElementMissing, None, None);
    };
    let pid = pid_for_element(focused.cast_mut());
    let name = pid.and_then(process_name);
    if reject_own_system_focus(pid, std::process::id() as i32, name.as_deref()) {
        unsafe { CFRelease(focused) };
        return (LiveAxOutcome::FocusedElementMissing, None, None);
    }
    let result = walk_selection(focused.cast_mut());
    unsafe { CFRelease(focused) };
    result
}

#[cfg(target_os = "macos")]
fn frontmost_pid() -> i32 {
    unsafe { crate::abi::bronze_native_frontmost_pid() }
}

#[cfg(target_os = "macos")]
pub fn note_external_focus() {
    use sys::remember_external_pid;
    remember_external_pid(frontmost_pid());
}

#[cfg(target_os = "macos")]
pub fn read_selection_for_pid(pid: i32) -> (LiveAxOutcome, Option<String>, Option<String>) {
    use sys::{
        is_own_pid, process_name, walk_application, AXUIElementCreateApplication, CFRelease,
    };
    if let Some(denied) = trusted_or_denied() {
        return denied;
    }
    if pid <= 0 || is_own_pid(pid) {
        return (LiveAxOutcome::FocusedElementMissing, None, None);
    }
    if process_name(pid).is_some_and(|name| is_skipped_process_name(&name)) {
        return (LiveAxOutcome::FocusedElementMissing, None, None);
    }
    let app = unsafe { AXUIElementCreateApplication(pid) };
    if app.is_null() {
        return (LiveAxOutcome::FocusedElementMissing, None, None);
    }
    sys::set_messaging_timeout(app);
    let result = walk_application(app);
    unsafe { CFRelease(app.cast()) };
    result
}

#[cfg(target_os = "macos")]
pub fn read_focused_selection() -> (LiveAxOutcome, Option<String>, Option<String>) {
    read_system_focused()
}

#[cfg(target_os = "macos")]
pub fn read_capture_selection() -> (LiveAxOutcome, Option<String>, Option<String>) {
    if let Some(pid) = last_external_pid() {
        return read_selection_for_pid(pid);
    }
    read_system_focused()
}

#[cfg(not(target_os = "macos"))]
pub fn read_focused_selection() -> (LiveAxOutcome, Option<String>, Option<String>) {
    (LiveAxOutcome::AccessibilityDenied, None, None)
}

#[cfg(not(target_os = "macos"))]
pub fn read_capture_selection() -> (LiveAxOutcome, Option<String>, Option<String>) {
    read_focused_selection()
}

#[cfg(not(target_os = "macos"))]
pub fn note_external_focus() {}

#[cfg(test)]
mod ax_live_tests {
    use super::*;

    #[test]
    fn live_ax_debug_never_includes_selection_body() {
        let outcome = LiveAxOutcome::Captured { len: 12 };
        assert!(!format!("{outcome:?}").contains("secret"));
        assert_eq!(LIVE_AX_MAX_BYTES, 1 << 20);
    }

    #[test]
    fn classify_roles_and_skip_bronze_process_names() {
        assert_eq!(
            classify_ax_role("AXTextArea", ""),
            AxProtection::AllowedText
        );
        assert_eq!(
            classify_ax_role("AXComboBox", ""),
            AxProtection::AllowedText
        );
        assert_eq!(classify_ax_role("AXWebArea", ""), AxProtection::AllowedText);
        assert_eq!(classify_ax_role("AXBrowser", ""), AxProtection::AllowedText);
        assert_eq!(
            classify_ax_role("AXDocument", ""),
            AxProtection::AllowedText
        );
        assert_eq!(
            classify_ax_role("AXWindow", ""),
            AxProtection::NeutralContainer
        );
        assert_eq!(
            classify_ax_role("AXSecureTextField", ""),
            AxProtection::Protected
        );
        assert_eq!(
            classify_ax_role("AXTextField", "AXSecureTextField"),
            AxProtection::Protected
        );
        assert_eq!(
            classify_ax_role("AXUnknown", ""),
            AxProtection::UnknownContentBearing
        );
        assert!(is_skipped_process_name("bronze-desktop"));
        assert!(is_skipped_process_name("Bronze"));
        assert!(!is_skipped_process_name("TextEdit"));
        assert_eq!(last_external_pid(), None);
        assert!(use_system_focused_fallback(None));
        assert!(!use_system_focused_fallback(Some(42)));
        assert!(reject_own_system_focus(Some(7), 7, Some("Cursor")));
        assert!(reject_own_system_focus(Some(99), 7, Some("bronze-desktop")));
        assert!(!reject_own_system_focus(Some(99), 7, Some("TextEdit")));
        let src = include_str!("ax_live.rs");
        let start = src
            .find("pub fn read_capture_selection()")
            .expect("read_capture_selection");
        let chunk = &src[start..];
        let end = chunk[1..]
            .find("\npub fn ")
            .map(|idx| idx + 1)
            .unwrap_or(chunk.len());
        assert!(
            !chunk[..end].contains("note_external_focus"),
            "persist read must keep the snapshotted last-external PID"
        );
        let take = src.find("fn take_owned").expect("take_owned");
        let take_fn = &src[take..src[take..].find("fn attr_string").expect("attr_string") + take];
        assert!(take_fn.contains("owned.take()"));
        let walk = src.find("pub fn walk_selection").expect("walk_selection");
        let walk_src = &src[walk..src.find("fn prefer_terminal").expect("prefer_terminal")];
        assert!(walk_src.contains("take_owned(&mut owned)"));
        assert!(walk_src.contains("None => break"));
        assert!(src.contains("AXAttributedStringForRange"));
        assert!(src.contains("AXUIElementSetMessagingTimeout"));
        assert!(src.contains("CFAttributedStringGetAttributes"));
        assert!(src.contains("set_messaging_timeout(system)"));
        assert!(src.contains("set_messaging_timeout(app)"));
        let tap = include_str!(
            "../../native/macos/BronzeNative/Sources/BronzeNative/EventTapEngine.swift"
        );
        assert!(!tap.contains("bronze_native_item_title"));
        assert!(!tap.contains("bronze_native_pasteboard_write"));
        assert!(!tap.contains("bronze_native_app_icon_png"));
        assert!(!tap.contains("bronze_native_bundle_id_for_pid"));
        assert!(!tap.contains("AXAttributedStringForRange"));
    }
}
