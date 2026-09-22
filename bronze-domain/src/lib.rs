//! bronze-domain scaffold (entities, commands, lifecycle per DAT-001)
//!
//! CAP-004: this crate has no macOS imports. Native access is only through
//! the platform façade crate.

mod composer;
mod entities;
mod formatters;
mod markup;
mod purge;
mod reflow;
mod title;
mod tokens;

pub use composer::{
    composer_should_add, ComposerChord, COMPOSER_ERROR_KEY, COMPOSER_LABEL_KEY, COMPOSER_SUBMIT_KEY,
};
pub use entities::{
    can_transition, ContentLanguage, InvalidTransition, Item, LanguageError, Lifecycle, Section,
};
pub use formatters::{
    default_output_profile, format_items, lifecycle_after_copy, AdvancePolicy, OutputFormat,
    OutputProfile, PostCopyAction,
};
pub use markup::{font_name_traits, html_from_constrained_markdown, markdown_from_runs, StyleRun};
pub use purge::{
    can_purge_with_descendants, claims_forensic_erasure, empty_trash_requires_confirmation,
    TRASH_RETENTION_DAYS,
};
pub use reflow::{
    check_reflow, composer_reachable, overflow_labeled, scaled_min_width, two_axis_scroll,
    ReflowError, Size, COMPOSER_MUST_BE_REACHABLE, REFLOW_WIDTH_CSS_PX, TEXT_RESIZE_PERCENT,
    TOOLBAR_OVERFLOW_KEY, TWO_AXIS_SCROLL_ALLOWED,
};
pub use title::{clamp_title, compact_title, strip_markup};
pub use tokens::{
    contrast_ratio, normal_text_pairs, relative_luminance, Rgb, ThemeTokens,
    COMPETITOR_TRADE_DRESS, CONCEPT_PNG_IS_PIXEL_SPEC, DARK, LIGHT, MIN_NORMAL_TEXT_CONTRAST,
};

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
