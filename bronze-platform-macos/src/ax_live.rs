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
        "AXTextField" | "AXTextArea" | "AXStaticText" | "AXWebArea" | "AXText" | "AXComboBox" => {
            AxProtection::AllowedText
        }
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

    pub fn take_text(value: CfTypeRef) -> Result<Option<String>, super::LiveAxOutcome> {
        let text = match cf_string_to_owned(value) {
            Ok(text) => text,
            Err(()) => {
                unsafe { CFRelease(value) };
                return Err(super::LiveAxOutcome::InvalidTextEncoding);
            }
        };
        unsafe { CFRelease(value) };
        if text.is_empty() {
            Ok(None)
        } else if text.len() > super::LIVE_AX_MAX_BYTES {
            Err(super::LiveAxOutcome::SelectionTooLarge)
        } else {
            Ok(Some(text))
        }
    }

    pub fn selected_text(element: AxUiElementRef) -> Result<Option<String>, super::LiveAxOutcome> {
        if let Some(selected) = copy_attr(element, "AXSelectedText") {
            if let Some(text) = take_text(selected)? {
                return Ok(Some(text));
            }
        }
        let Some(range) = copy_attr(element, "AXSelectedTextRange") else {
            return Ok(None);
        };
        let parameterized = copy_param(element, "AXStringForRange", range);
        unsafe { CFRelease(range) };
        match parameterized {
            Some(value) => take_text(value),
            None => Ok(None),
        }
    }

    pub fn walk_selection(
        start: AxUiElementRef,
    ) -> (super::LiveAxOutcome, Option<String>, Option<String>) {
        let source_app_name = app_name_for_element(start);
        let mut current = start;
        let mut owned: Option<CfTypeRef> = None;
        for _ in 0..super::AX_CHAIN_LIMIT {
            let role = copy_attr(current, "AXRole")
                .and_then(|role| cf_string_to_owned(role).ok())
                .unwrap_or_default();
            let subrole = copy_attr(current, "AXSubrole")
                .and_then(|role| cf_string_to_owned(role).ok())
                .unwrap_or_default();
            match super::classify_ax_role(&role, &subrole) {
                super::AxProtection::Protected => {
                    if let Some(prev) = owned {
                        unsafe { CFRelease(prev) };
                    }
                    return (super::LiveAxOutcome::ProtectedContent, None, None);
                }
                super::AxProtection::AllowedText => match selected_text(current) {
                    Ok(Some(text)) => {
                        if let Some(prev) = owned {
                            unsafe { CFRelease(prev) };
                        }
                        let len = text.len();
                        return (
                            super::LiveAxOutcome::Captured { len },
                            Some(text),
                            source_app_name,
                        );
                    }
                    Err(outcome) => {
                        if let Some(prev) = owned {
                            unsafe { CFRelease(prev) };
                        }
                        return (outcome, None, None);
                    }
                    Ok(None) => {}
                },
                super::AxProtection::NeutralContainer
                | super::AxProtection::UnknownContentBearing => {}
            }
            let parent = copy_attr(current, "AXParent");
            if let Some(prev) = owned {
                unsafe { CFRelease(prev) };
            }
            match parent {
                Some(next) => {
                    owned = Some(next);
                    current = next.cast_mut();
                }
                None => break,
            }
        }
        if let Some(prev) = owned {
            unsafe { CFRelease(prev) };
        }
        (super::LiveAxOutcome::NoSelection, None, source_app_name)
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
    use sys::{copy_attr, walk_selection, AXUIElementCreateSystemWide, CFRelease};
    if let Some(denied) = trusted_or_denied() {
        return denied;
    }
    let system = unsafe { AXUIElementCreateSystemWide() };
    if system.is_null() {
        return (LiveAxOutcome::FocusedElementMissing, None, None);
    }
    let focused = copy_attr(system, "AXFocusedUIElement");
    unsafe { CFRelease(system.cast()) };
    let Some(focused) = focused else {
        return (LiveAxOutcome::FocusedElementMissing, None, None);
    };
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
        copy_attr, is_own_pid, process_name, walk_selection, AXUIElementCreateApplication,
        CFRelease,
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
    let focused = copy_attr(app, "AXFocusedUIElement");
    unsafe { CFRelease(app.cast()) };
    let Some(focused) = focused else {
        return (LiveAxOutcome::FocusedElementMissing, None, None);
    };
    let result = walk_selection(focused.cast_mut());
    unsafe { CFRelease(focused) };
    result
}

#[cfg(target_os = "macos")]
pub fn read_focused_selection() -> (LiveAxOutcome, Option<String>, Option<String>) {
    read_system_focused()
}

#[cfg(target_os = "macos")]
pub fn read_capture_selection() -> (LiveAxOutcome, Option<String>, Option<String>) {
    note_external_focus();
    if let Some(pid) = last_external_pid() {
        let result = read_selection_for_pid(pid);
        match result.0 {
            LiveAxOutcome::Captured { .. }
            | LiveAxOutcome::ProtectedContent
            | LiveAxOutcome::ProtectionUnknown
            | LiveAxOutcome::SelectionTooLarge
            | LiveAxOutcome::InvalidTextEncoding
            | LiveAxOutcome::AccessibilityDenied => return result,
            LiveAxOutcome::NoSelection | LiveAxOutcome::FocusedElementMissing => {}
        }
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
    }
}
