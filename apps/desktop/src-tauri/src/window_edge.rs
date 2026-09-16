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
pub const QUICK_PANEL_WIDTH: u32 = 360;
pub const QUICK_PANEL_HEIGHT: u32 = 640;
pub const DEV_LAUNCH_REVEALS_QUICK: bool = true;
pub const CAPTURE_ONLY_REVEALS_PANEL: bool = false;
pub const DEFAULT_PHYSICAL_EDGE: PhysicalEdge = PhysicalEdge::Left;

const _: () = assert!(DEV_LAUNCH_REVEALS_QUICK);
const _: () = assert!(!CAPTURE_ONLY_REVEALS_PANEL);

pub fn allowed_chrome_window(kind: &str) -> Option<&'static str> {
    match kind {
        "library" => Some("library"),
        "settings" => Some("settings"),
        _ => None,
    }
}

pub fn quick_panel_frame(work: Rect, dir: TextDirection) -> Rect {
    place_on_physical_edge(
        DEFAULT_PHYSICAL_EDGE,
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
        width: 360,
        height: 640,
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
        assert_eq!(right.x, WORK.x + WORK.width as i32 - 360);
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
                width: 360,
                height: 640,
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
        assert_eq!(allowed_chrome_window("quick"), None);
        assert_eq!(allowed_chrome_window("onboarding"), None);
        assert_eq!(allowed_chrome_window(""), None);
    }

    #[test]
    fn dev_launch_reveals_quick_on_physical_edge_without_capture_steal() {
        assert!(DEV_LAUNCH_REVEALS_QUICK);
        assert!(!CAPTURE_ONLY_REVEALS_PANEL);
        let frame = quick_panel_frame(WORK, TextDirection::Ltr);
        let rtl = quick_panel_frame(WORK, TextDirection::Rtl);
        assert_eq!(frame, rtl);
        assert_eq!(frame.x, WORK.x);
        assert_eq!(frame.width, QUICK_PANEL_WIDTH);
        assert_eq!(frame.height, WORK.height);
        assert_eq!(DEFAULT_PHYSICAL_EDGE, PhysicalEdge::Left);
    }
}
