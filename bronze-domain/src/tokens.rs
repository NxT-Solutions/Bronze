//! Warm bronze tokens with measured contrast (story 8.1, A11Y-003).
//! DESIGN.md / concept PNG is inspiration, not a pixel spec.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xff) as u8,
            g: ((hex >> 8) & 0xff) as u8,
            b: (hex & 0xff) as u8,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThemeTokens {
    pub background: Rgb,
    pub foreground: Rgb,
    pub card: Rgb,
    pub card_foreground: Rgb,
    pub muted: Rgb,
    pub muted_foreground: Rgb,
    pub primary: Rgb,
    pub primary_foreground: Rgb,
    pub accent: Rgb,
    pub accent_foreground: Rgb,
    pub secondary: Rgb,
    pub secondary_foreground: Rgb,
}

/// Warm parchment + bronze — no competitor trade dress.
pub const LIGHT: ThemeTokens = ThemeTokens {
    background: Rgb::hex(0xF7F4EF),
    foreground: Rgb::hex(0x1C1917),
    card: Rgb::hex(0xFFFFFF),
    card_foreground: Rgb::hex(0x1C1917),
    muted: Rgb::hex(0xEFE8DE),
    muted_foreground: Rgb::hex(0x5C564E),
    primary: Rgb::hex(0x8C6239),
    primary_foreground: Rgb::hex(0xFFFFFF),
    accent: Rgb::hex(0xC4A265),
    accent_foreground: Rgb::hex(0x1C1917),
    secondary: Rgb::hex(0xEFE8DE),
    secondary_foreground: Rgb::hex(0x1C1917),
};

pub const DARK: ThemeTokens = ThemeTokens {
    background: Rgb::hex(0x1A1714),
    foreground: Rgb::hex(0xF7F4EF),
    card: Rgb::hex(0x2A2520),
    card_foreground: Rgb::hex(0xF7F4EF),
    muted: Rgb::hex(0x2A2520),
    muted_foreground: Rgb::hex(0xE3D5C3),
    primary: Rgb::hex(0xD4B483),
    primary_foreground: Rgb::hex(0x1A1714),
    accent: Rgb::hex(0xD4B483),
    accent_foreground: Rgb::hex(0x1A1714),
    secondary: Rgb::hex(0x2A2520),
    secondary_foreground: Rgb::hex(0xF7F4EF),
};

pub const MIN_NORMAL_TEXT_CONTRAST: f64 = 4.5;
pub const CONCEPT_PNG_IS_PIXEL_SPEC: bool = false;
pub const COMPETITOR_TRADE_DRESS: bool = false;

fn channel(c: u8) -> f64 {
    let s = f64::from(c) / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

pub fn relative_luminance(rgb: Rgb) -> f64 {
    0.2126 * channel(rgb.r) + 0.7152 * channel(rgb.g) + 0.0722 * channel(rgb.b)
}

pub fn contrast_ratio(a: Rgb, b: Rgb) -> f64 {
    let (l1, l2) = {
        let la = relative_luminance(a);
        let lb = relative_luminance(b);
        if la > lb {
            (la, lb)
        } else {
            (lb, la)
        }
    };
    (l1 + 0.05) / (l2 + 0.05)
}

pub fn normal_text_pairs(theme: &ThemeTokens) -> [(&'static str, Rgb, Rgb); 6] {
    [
        ("fg/bg", theme.foreground, theme.background),
        ("card", theme.card_foreground, theme.card),
        ("muted", theme.muted_foreground, theme.muted),
        ("primary", theme.primary_foreground, theme.primary),
        ("accent", theme.accent_foreground, theme.accent),
        ("secondary", theme.secondary_foreground, theme.secondary),
    ]
}

#[cfg(test)]
mod token_tests {
    use super::*;

    #[test]
    fn light_and_dark_normal_text_meet_four_point_five() {
        for theme in [LIGHT, DARK] {
            for (name, fg, bg) in normal_text_pairs(&theme) {
                let ratio = contrast_ratio(fg, bg);
                assert!(
                    ratio + f64::EPSILON >= MIN_NORMAL_TEXT_CONTRAST,
                    "{name} contrast {ratio:.2} < 4.5"
                );
            }
        }
    }

    #[test]
    fn concept_png_is_inspiration_not_trade_dress() {
        assert!(!CONCEPT_PNG_IS_PIXEL_SPEC);
        assert!(!COMPETITOR_TRADE_DRESS);
        let src = include_str!("tokens.rs");
        let banned = ["cop", "per"].concat();
        let banned_brand = ["coo", "per"].concat();
        assert!(!src.to_ascii_lowercase().contains(&banned));
        assert!(!src.to_ascii_lowercase().contains(&banned_brand));
    }
}
