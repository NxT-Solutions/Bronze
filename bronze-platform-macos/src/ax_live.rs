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
pub fn read_focused_selection() -> (LiveAxOutcome, Option<String>) {
    use sys::{
        cf_string_to_owned, copy_attr, AXIsProcessTrusted, AXUIElementCreateSystemWide, CFRelease,
    };

    if unsafe { AXIsProcessTrusted() } == 0 {
        return (LiveAxOutcome::AccessibilityDenied, None);
    }
    let system = unsafe { AXUIElementCreateSystemWide() };
    if system.is_null() {
        return (LiveAxOutcome::FocusedElementMissing, None);
    }
    let focused = copy_attr(system, "AXFocusedUIElement");
    unsafe { CFRelease(system.cast()) };
    let Some(focused) = focused else {
        return (LiveAxOutcome::FocusedElementMissing, None);
    };
    let focused_el = focused.cast_mut();
    let role = copy_attr(focused_el, "AXRole")
        .and_then(|role| cf_string_to_owned(role).ok())
        .unwrap_or_default();
    if role == "AXSecureTextField" {
        unsafe { CFRelease(focused) };
        return (LiveAxOutcome::ProtectedContent, None);
    }
    if role == "AXUnknown" {
        unsafe { CFRelease(focused) };
        return (LiveAxOutcome::ProtectionUnknown, None);
    }
    let selected = copy_attr(focused_el, "AXSelectedText");
    unsafe { CFRelease(focused) };
    let Some(selected) = selected else {
        return (LiveAxOutcome::NoSelection, None);
    };
    let text = match cf_string_to_owned(selected) {
        Ok(text) => text,
        Err(()) => {
            unsafe { CFRelease(selected) };
            return (LiveAxOutcome::InvalidTextEncoding, None);
        }
    };
    unsafe { CFRelease(selected) };
    if text.is_empty() {
        return (LiveAxOutcome::NoSelection, None);
    }
    if text.len() > LIVE_AX_MAX_BYTES {
        return (LiveAxOutcome::SelectionTooLarge, None);
    }
    let len = text.len();
    (LiveAxOutcome::Captured { len }, Some(text))
}

#[cfg(not(target_os = "macos"))]
pub fn read_focused_selection() -> (LiveAxOutcome, Option<String>) {
    (LiveAxOutcome::AccessibilityDenied, None)
}

#[cfg(test)]
mod ax_live_tests {
    use super::*;

    #[test]
    fn live_ax_debug_never_includes_selection_body() {
        let outcome = LiveAxOutcome::Captured { len: 12 };
        assert!(!format!("{outcome:?}").contains("secret"));
        assert_eq!(LIVE_AX_MAX_BYTES, 1 << 20);
    }
}
