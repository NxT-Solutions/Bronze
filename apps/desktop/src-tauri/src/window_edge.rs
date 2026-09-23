//! Physical-edge quick panel (story 5.2, WIN-001/002/005, ADR-007).
//! Stored edge is physical left/right/top; RTL never mirrors it.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhysicalEdge {
    Left,
    Right,
    Top,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EdgeError {
    RtlRelativeForbidden,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextDirection {
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PanelKind {
    Activating,
    Nonactivating,
}

pub const PANEL_KIND: PanelKind = PanelKind::Activating;
pub const FOCUS_TRAP: bool = false;
pub const QUICK_PANEL_WIDTH: u32 = 400;
pub const QUICK_PANEL_HEIGHT: u32 = 720;
pub const QUICK_PANEL_MIN_WIDTH: u32 = bronze_domain::REFLOW_WIDTH_CSS_PX;
pub const QUICK_PANEL_MIN_HEIGHT: u32 = 560;
pub const DEV_LAUNCH_REVEALS_QUICK: bool = true;
pub const CAPTURE_ONLY_REVEALS_PANEL: bool = false;
pub const DEFAULT_PHYSICAL_EDGE: PhysicalEdge = PhysicalEdge::Left;

const _: () = assert!(DEV_LAUNCH_REVEALS_QUICK);
const _: () = assert!(!CAPTURE_ONLY_REVEALS_PANEL);

pub fn physical_rect_to_logical(physical: Rect, scale_factor: f64) -> Rect {
    let scale = if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };
    Rect {
        x: (f64::from(physical.x) / scale).round() as i32,
        y: (f64::from(physical.y) / scale).round() as i32,
        width: (f64::from(physical.width) / scale).round() as u32,
        height: (f64::from(physical.height) / scale).round() as u32,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChromeWindowSpec {
    pub label: &'static str,
    pub url: &'static str,
    pub title: &'static str,
    pub width: f64,
    pub height: f64,
    pub min_width: f64,
    pub min_height: f64,
}

pub fn allowed_chrome_window(kind: &str) -> Option<&'static str> {
    chrome_window_spec(kind).map(|spec| spec.label)
}

pub fn chrome_window_spec(kind: &str) -> Option<ChromeWindowSpec> {
    match kind {
        "library" => Some(ChromeWindowSpec {
            label: "library",
            url: "library.html",
            title: "Bronze Library",
            width: 720.0,
            height: 640.0,
            min_width: 480.0,
            min_height: 400.0,
        }),
        "settings" => Some(ChromeWindowSpec {
            label: "settings",
            url: "settings.html",
            title: "Bronze Settings",
            width: 640.0,
            height: 720.0,
            min_width: 480.0,
            min_height: 480.0,
        }),
        "help" => Some(ChromeWindowSpec {
            label: "help",
            url: "help.html",
            title: "Bronze Help",
            width: 640.0,
            height: 640.0,
            min_width: 480.0,
            min_height: 400.0,
        }),
        _ => None,
    }
}

pub fn quick_panel_frame(edge: PhysicalEdge, work: Rect, dir: TextDirection) -> Rect {
    place_on_physical_edge(
        edge,
        Rect {
            x: 0,
            y: 0,
            width: QUICK_PANEL_WIDTH,
            height: QUICK_PANEL_HEIGHT,
        },
        work,
        dir,
    )
}

pub fn parse_physical_edge(raw: &str) -> Result<PhysicalEdge, EdgeError> {
    match raw {
        "left" => Ok(PhysicalEdge::Left),
        "right" => Ok(PhysicalEdge::Right),
        "top" => Ok(PhysicalEdge::Top),
        "leading" | "trailing" | "start" | "end" | "inline-start" | "inline-end" => {
            Err(EdgeError::RtlRelativeForbidden)
        }
        _ => Err(EdgeError::Unknown),
    }
}

pub fn stored_edge_ignores_direction(edge: PhysicalEdge, _dir: TextDirection) -> PhysicalEdge {
    edge
}

pub fn clamp_to_work_area(panel: Rect, work: Rect) -> Rect {
    let width = panel.width.min(work.width);
    let height = panel.height.min(work.height);
    let max_x = work.x + work.width as i32 - width as i32;
    let max_y = work.y + work.height as i32 - height as i32;
    Rect {
        x: panel.x.clamp(work.x, max_x.max(work.x)),
        y: panel.y.clamp(work.y, max_y.max(work.y)),
        width,
        height,
    }
}

pub fn place_on_physical_edge(
    edge: PhysicalEdge,
    preferred: Rect,
    work: Rect,
    dir: TextDirection,
) -> Rect {
    let edge = stored_edge_ignores_direction(edge, dir);
    let width = preferred.width.min(work.width);
    let height = preferred.height.min(work.height);
    let placed = match edge {
        PhysicalEdge::Left => Rect {
            x: work.x,
            y: work.y,
            width,
            height: work.height,
        },
        PhysicalEdge::Right => Rect {
            x: work.x + work.width as i32 - width as i32,
            y: work.y,
            width,
            height: work.height,
        },
        PhysicalEdge::Top => Rect {
            x: work.x,
            y: work.y,
            width: work.width,
            height,
        },
    };
    clamp_to_work_area(placed, work)
}

#[cfg(test)]
mod window_edge_tests {
    use super::*;

    const WORK: Rect = Rect {
        x: 100,
        y: 50,
        width: 1000,
        height: 700,
    };
    const PANEL: Rect = Rect {
        x: 0,
        y: 0,
        width: QUICK_PANEL_WIDTH,
        height: QUICK_PANEL_HEIGHT,
    };

    #[test]
    fn window_edge_is_physical_not_rtl_leading() {
        assert_eq!(parse_physical_edge("left"), Ok(PhysicalEdge::Left));
        assert_eq!(parse_physical_edge("right"), Ok(PhysicalEdge::Right));
        assert_eq!(parse_physical_edge("top"), Ok(PhysicalEdge::Top));
        assert_eq!(
            parse_physical_edge("leading"),
            Err(EdgeError::RtlRelativeForbidden)
        );
        assert_eq!(
            parse_physical_edge("start"),
            Err(EdgeError::RtlRelativeForbidden)
        );
        let left_ltr = place_on_physical_edge(PhysicalEdge::Left, PANEL, WORK, TextDirection::Ltr);
        let left_rtl = place_on_physical_edge(PhysicalEdge::Left, PANEL, WORK, TextDirection::Rtl);
        assert_eq!(left_ltr, left_rtl);
        assert_eq!(left_ltr.x, WORK.x);
        let right = place_on_physical_edge(PhysicalEdge::Right, PANEL, WORK, TextDirection::Rtl);
        assert_eq!(
            right.x,
            WORK.x + WORK.width as i32 - QUICK_PANEL_WIDTH as i32
        );
        assert_eq!(
            stored_edge_ignores_direction(PhysicalEdge::Left, TextDirection::Rtl),
            PhysicalEdge::Left
        );
    }

    #[test]
    fn window_edge_clamps_to_work_area() {
        let overflow = Rect {
            x: 2000,
            y: -40,
            width: 2000,
            height: 2000,
        };
        let clamped = clamp_to_work_area(overflow, WORK);
        assert_eq!(clamped.width, WORK.width);
        assert_eq!(clamped.height, WORK.height);
        assert_eq!(clamped.x, WORK.x);
        assert_eq!(clamped.y, WORK.y);
        let shrunk = Rect {
            x: 100,
            y: 50,
            width: 400,
            height: 300,
        };
        let after = clamp_to_work_area(
            Rect {
                x: 100,
                y: 50,
                width: QUICK_PANEL_WIDTH,
                height: QUICK_PANEL_HEIGHT,
            },
            shrunk,
        );
        assert_eq!(after.height, 300);
        assert!(after.x + after.width as i32 <= shrunk.x + shrunk.width as i32);
    }

    #[test]
    fn window_edge_panel_is_activating_without_focus_trap() {
        assert_eq!(PANEL_KIND, PanelKind::Activating);
        assert_ne!(PANEL_KIND, PanelKind::Nonactivating);
        assert!(!FOCUS_TRAP);
    }

    #[test]
    fn chrome_window_allow_list_excludes_arbitrary_labels() {
        assert_eq!(allowed_chrome_window("library"), Some("library"));
        assert_eq!(allowed_chrome_window("settings"), Some("settings"));
        assert_eq!(allowed_chrome_window("help"), Some("help"));
        assert_eq!(allowed_chrome_window("quick"), None);
        assert_eq!(allowed_chrome_window("onboarding"), None);
        assert_eq!(allowed_chrome_window(""), None);
        let settings = chrome_window_spec("settings").expect("settings spec");
        assert_eq!(settings.url, "settings.html");
        assert_eq!(settings.min_width, 480.0);
        assert!(chrome_window_spec("quick").is_none());
    }

    #[test]
    fn dev_launch_reveals_quick_on_physical_edge_without_capture_steal() {
        assert!(DEV_LAUNCH_REVEALS_QUICK);
        assert!(!CAPTURE_ONLY_REVEALS_PANEL);
        let frame = quick_panel_frame(DEFAULT_PHYSICAL_EDGE, WORK, TextDirection::Ltr);
        let rtl = quick_panel_frame(DEFAULT_PHYSICAL_EDGE, WORK, TextDirection::Rtl);
        assert_eq!(frame, rtl);
        assert_eq!(frame.x, WORK.x);
        assert_eq!(frame.width, QUICK_PANEL_WIDTH);
        assert_eq!(frame.height, WORK.height);
        assert_eq!(DEFAULT_PHYSICAL_EDGE, PhysicalEdge::Left);
    }

    #[test]
    fn quick_panel_defaults_are_comfortable_above_reflow_floor() {
        assert_eq!(QUICK_PANEL_MIN_WIDTH, bronze_domain::REFLOW_WIDTH_CSS_PX);
        assert_eq!(QUICK_PANEL_WIDTH, 400);
        assert_eq!(QUICK_PANEL_HEIGHT, 720);
        assert_eq!(QUICK_PANEL_MIN_HEIGHT, 560);
        assert!(QUICK_PANEL_WIDTH > QUICK_PANEL_MIN_WIDTH);
        assert!(QUICK_PANEL_HEIGHT > QUICK_PANEL_MIN_HEIGHT);
    }

    #[test]
    fn work_area_physical_pixels_convert_to_logical_css_px() {
        let retina = Rect {
            x: 0,
            y: 74,
            width: 3024,
            height: 1890,
        };
        let logical = physical_rect_to_logical(retina, 2.0);
        assert_eq!(
            logical,
            Rect {
                x: 0,
                y: 37,
                width: 1512,
                height: 945,
            }
        );
        let frame = quick_panel_frame(DEFAULT_PHYSICAL_EDGE, logical, TextDirection::Ltr);
        assert_eq!(frame.width, QUICK_PANEL_WIDTH);
        assert_eq!(frame.height, logical.height);
        assert!(frame.width > QUICK_PANEL_MIN_WIDTH);
        let unscaled_design = physical_rect_to_logical(
            Rect {
                x: 0,
                y: 0,
                width: QUICK_PANEL_WIDTH,
                height: QUICK_PANEL_HEIGHT,
            },
            2.0,
        );
        assert!(unscaled_design.width < QUICK_PANEL_MIN_WIDTH);
        assert_eq!(physical_rect_to_logical(retina, 0.0).width, retina.width);
    }
}
