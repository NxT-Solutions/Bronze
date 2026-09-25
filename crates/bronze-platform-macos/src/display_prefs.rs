//! Native display-preference snapshot (story 5.4, A11Y-003, WIN-004).
//! App overrides may only strengthen active OS flags.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DisplayPreferenceSnapshot {
    pub reduce_motion: bool,
    pub reduce_transparency: bool,
    pub increase_contrast: bool,
    pub differentiate_without_color: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisplayPrefError {
    WeakenForbidden,
}

pub trait DisplayPrefHost {
    fn read(&self) -> DisplayPreferenceSnapshot;
}

#[derive(Clone, Copy, Debug)]
pub struct FakeDisplayHost {
    pub system: DisplayPreferenceSnapshot,
}

impl DisplayPrefHost for FakeDisplayHost {
    fn read(&self) -> DisplayPreferenceSnapshot {
        self.system
    }
}

pub fn strengthen(
    system: DisplayPreferenceSnapshot,
    app: DisplayPreferenceSnapshot,
) -> Result<DisplayPreferenceSnapshot, DisplayPrefError> {
    Ok(DisplayPreferenceSnapshot {
        reduce_motion: system.reduce_motion || app.reduce_motion,
        reduce_transparency: system.reduce_transparency || app.reduce_transparency,
        increase_contrast: system.increase_contrast || app.increase_contrast,
        differentiate_without_color: system.differentiate_without_color
            || app.differentiate_without_color,
    })
}

pub struct DisplayPrefBridge<H> {
    host: H,
    app: DisplayPreferenceSnapshot,
    last: Option<DisplayPreferenceSnapshot>,
}

impl<H: DisplayPrefHost> DisplayPrefBridge<H> {
    pub fn new(host: H) -> Self {
        Self {
            host,
            app: DisplayPreferenceSnapshot {
                reduce_motion: false,
                reduce_transparency: false,
                increase_contrast: false,
                differentiate_without_color: false,
            },
            last: None,
        }
    }

    pub fn set_app_override(
        &mut self,
        app: DisplayPreferenceSnapshot,
    ) -> Result<DisplayPreferenceSnapshot, DisplayPrefError> {
        self.app = app;
        self.publish()
    }

    pub fn publish(&mut self) -> Result<DisplayPreferenceSnapshot, DisplayPrefError> {
        let snapshot = strengthen(self.host.read(), self.app)?;
        self.last = Some(snapshot);
        Ok(snapshot)
    }

    pub fn last_published(&self) -> Option<DisplayPreferenceSnapshot> {
        self.last
    }
}

#[cfg(test)]
mod display_prefs_tests {
    use super::*;

    fn off() -> DisplayPreferenceSnapshot {
        DisplayPreferenceSnapshot {
            reduce_motion: false,
            reduce_transparency: false,
            increase_contrast: false,
            differentiate_without_color: false,
        }
    }

    fn motion_on() -> DisplayPreferenceSnapshot {
        DisplayPreferenceSnapshot {
            reduce_motion: true,
            ..off()
        }
    }

    #[test]
    fn display_prefs_publishes_one_typed_snapshot() {
        let host = FakeDisplayHost {
            system: DisplayPreferenceSnapshot {
                reduce_motion: true,
                reduce_transparency: true,
                increase_contrast: false,
                differentiate_without_color: true,
            },
        };
        let mut bridge = DisplayPrefBridge::new(host);
        let snap = bridge.publish().expect("publish");
        assert_eq!(bridge.last_published(), Some(snap));
        assert!(snap.reduce_motion);
        assert!(snap.reduce_transparency);
        assert!(!snap.increase_contrast);
        assert!(snap.differentiate_without_color);
    }

    #[test]
    fn display_prefs_overrides_only_strengthen() {
        let host = FakeDisplayHost { system: off() };
        let mut bridge = DisplayPrefBridge::new(host);
        let snap = bridge.set_app_override(motion_on()).expect("strengthen");
        assert!(snap.reduce_motion);
        let system_on = FakeDisplayHost {
            system: motion_on(),
        };
        let mut live = DisplayPrefBridge::new(system_on);
        let weakened = DisplayPreferenceSnapshot {
            reduce_motion: false,
            ..off()
        };
        let still_on = live.set_app_override(weakened).expect("cannot weaken");
        assert!(still_on.reduce_motion);
        assert_eq!(strengthen(motion_on(), off()).expect("or"), motion_on());
    }

    #[test]
    fn display_prefs_tokens_update_without_restart() {
        let host = FakeDisplayHost { system: off() };
        let mut bridge = DisplayPrefBridge::new(host);
        let first = bridge.publish().expect("first");
        assert!(!first.reduce_motion);
        bridge.host.system = motion_on();
        let second = bridge.publish().expect("live");
        assert!(second.reduce_motion);
        assert_ne!(first, second);
        assert_eq!(bridge.last_published(), Some(second));
    }
}
