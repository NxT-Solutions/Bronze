//! bronze-domain scaffold (entities, commands, lifecycle per DAT-001)
//!
//! CAP-004: this crate has no macOS imports. Native access is only through
//! the platform façade crate.

#[cfg(test)]
mod tests {
    #[test]
    fn domain_crate_has_no_native_or_macos_deps() {
        let manifest = include_str!("../Cargo.toml");
        assert!(
            !manifest.contains("[dependencies]"),
            "bronze-domain must not declare crate dependencies (CAP-004)"
        );
        let src = include_str!("lib.rs");
        let extern_crate = ["extern", " crate"].concat();
        assert!(!src.contains(&extern_crate));
        assert!(!src.contains(&["use bronze_", "platform"].concat()));
        assert!(!src.contains(&["use ", "objc"].concat()));
        assert!(!src.contains(&["use ", "AppKit"].concat()));
    }
}
