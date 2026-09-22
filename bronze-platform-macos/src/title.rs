use crate::abi::{BronzeNativeUtf8View, BRONZE_STATUS_OK};

pub fn native_item_title(body: &str) -> Option<String> {
    if body.is_empty() {
        return None;
    }
    let view = BronzeNativeUtf8View {
        ptr: body.as_ptr(),
        len: body.len() as u64,
    };
    let mut out = BronzeNativeUtf8View {
        ptr: std::ptr::null(),
        len: 0,
    };
    let status = unsafe { crate::abi::bronze_native_item_title(view, &mut out) };
    if status != BRONZE_STATUS_OK {
        return None;
    }
    let copied = if out.ptr.is_null() {
        (out.len == 0).then(String::new)
    } else {
        let slice = unsafe { std::slice::from_raw_parts(out.ptr, out.len as usize) };
        String::from_utf8(slice.to_vec()).ok()
    };
    let _ = unsafe { crate::abi::bronze_native_utf8_free(out) };
    copied.filter(|text| !text.trim().is_empty())
}

#[cfg(test)]
mod title_tests {
    use super::*;

    #[test]
    fn stub_or_unavailable_model_returns_none_without_body() {
        assert_eq!(native_item_title(""), None);
        assert_eq!(
            native_item_title("The migration timeout is the real bug in persist."),
            None
        );
        assert!(!format!("{:?}", native_item_title("secret-title-body")).contains("secret"));
    }

    #[test]
    fn title_abi_has_no_vendor_or_apple_model() {
        let src =
            include_str!("../../native/macos/BronzeNative/Sources/BronzeNative/TitleABI.swift");
        let lower = src.to_ascii_lowercase();
        for needle in [
            "privatecloudcompute",
            "urlsession",
            "openai",
            "llama",
            "gguf",
            "systemlanguagemodel",
            "nlembedding",
            "foundationmodels",
            "naturallanguage",
            "huggingface",
            "qwen",
            "cactus",
            "needle",
        ] {
            assert!(!lower.contains(needle), "TitleABI mentions {needle}");
        }
        assert!(src.contains("BRONZE_STATUS_DEGRADED"));
        assert!(!src.contains("resolveTitle"));
    }
}
