//! Reflow contract at 320 CSS px and 200% text (story 8.2, A11Y-003).

pub const REFLOW_WIDTH_CSS_PX: u32 = 320;
pub const TEXT_RESIZE_PERCENT: u32 = 200;
pub const TWO_AXIS_SCROLL_ALLOWED: bool = false;
pub const TOOLBAR_OVERFLOW_KEY: &str = "panel.toolbar.overflow";
pub const COMPOSER_MUST_BE_REACHABLE: bool = true;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReflowError {
    TwoAxisScroll,
    UnlabeledOverflow,
    ComposerUnreachable,
}

pub fn scaled_min_width(base_css_px: u32, text_percent: u32) -> u32 {
    base_css_px.saturating_mul(text_percent) / 100
}

pub fn two_axis_scroll(content: Size, viewport: Size) -> bool {
    content.width > viewport.width && content.height > viewport.height
}

pub fn overflow_labeled(accessible_name: Option<&str>) -> bool {
    accessible_name
        .map(|name| !name.is_empty())
        .unwrap_or(false)
}

pub fn composer_reachable(composer_in_flow: bool, tab_index: i32) -> bool {
    COMPOSER_MUST_BE_REACHABLE && composer_in_flow && tab_index >= 0
}

pub fn check_reflow(
    content: Size,
    viewport: Size,
    overflow_name: Option<&str>,
    composer_in_flow: bool,
) -> Result<(), ReflowError> {
    if two_axis_scroll(content, viewport) {
        return Err(ReflowError::TwoAxisScroll);
    }
    if viewport.width <= REFLOW_WIDTH_CSS_PX && !overflow_labeled(overflow_name) {
        return Err(ReflowError::UnlabeledOverflow);
    }
    if !composer_reachable(composer_in_flow, 0) {
        return Err(ReflowError::ComposerUnreachable);
    }
    Ok(())
}

#[cfg(test)]
mod reflow_tests {
    use super::*;

    #[test]
    fn three_twenty_and_two_hundred_forbid_two_axis_scroll() {
        assert_eq!(REFLOW_WIDTH_CSS_PX, 320);
        assert_eq!(TEXT_RESIZE_PERCENT, 200);
        assert!(!TWO_AXIS_SCROLL_ALLOWED);
        let viewport = Size {
            width: REFLOW_WIDTH_CSS_PX,
            height: 480,
        };
        let stacked = Size {
            width: REFLOW_WIDTH_CSS_PX,
            height: 900,
        };
        assert!(!two_axis_scroll(stacked, viewport));
        let both = Size {
            width: 400,
            height: 900,
        };
        assert!(two_axis_scroll(both, viewport));
        assert_eq!(
            check_reflow(both, viewport, Some("More"), true),
            Err(ReflowError::TwoAxisScroll)
        );
        assert_eq!(
            check_reflow(stacked, viewport, None, true),
            Err(ReflowError::UnlabeledOverflow)
        );
        assert_eq!(
            check_reflow(stacked, viewport, Some("More actions"), true),
            Ok(())
        );
        assert_eq!(
            check_reflow(stacked, viewport, Some("More"), false),
            Err(ReflowError::ComposerUnreachable)
        );
        assert_eq!(TOOLBAR_OVERFLOW_KEY, "panel.toolbar.overflow");
    }
}
