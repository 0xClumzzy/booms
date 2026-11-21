//! Styling: foreground/background colors and text attributes.

use std::ops::BitOr;

/// A text color or background color.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Color {
    /// The terminal's default color.
    #[default]
    Default,
    /// One of the 16 standard ANSI colors (`n` in `0..=15`).
    Ansi(u8),
    /// One of the 256 terminal colors.
    Ansi256(u8),
    /// A 24-bit true color.
    Rgb(u8, u8, u8),
}

impl Color {
    /// Build a [`Color::Rgb`].
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color::Rgb(r, g, b)
    }

    /// The RGB channels, if this is [`Color::Rgb`].
    pub const fn channels(&self) -> Option<(u8, u8, u8)> {
        match self {
            Color::Rgb(r, g, b) => Some((*r, *g, *b)),
            _ => None,
        }
    }
}

/// Text attributes, as a bitset.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Modifiers {
    bits: u8,
}

impl Modifiers {
    pub const NONE: Modifiers = Modifiers { bits: 0 };
    pub const BOLD: Modifiers = Modifiers { bits: 1 << 0 };
    pub const DIM: Modifiers = Modifiers { bits: 1 << 1 };
    pub const ITALIC: Modifiers = Modifiers { bits: 1 << 2 };
    pub const UNDERLINE: Modifiers = Modifiers { bits: 1 << 3 };
    pub const BLINK: Modifiers = Modifiers { bits: 1 << 4 };
    pub const REVERSE: Modifiers = Modifiers { bits: 1 << 5 };
    pub const STRIKETHROUGH: Modifiers = Modifiers { bits: 1 << 6 };

    /// Union of the flags in `self` and `other`.
    pub const fn union(self, other: Modifiers) -> Modifiers {
        Modifiers {
            bits: self.bits | other.bits,
        }
    }

    /// True if every flag in `other` is also set in `self`.
    pub const fn contains(self, other: Modifiers) -> bool {
        self.bits & other.bits == other.bits
    }

    /// True if no flag is set.
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }
}

impl BitOr for Modifiers {
    type Output = Modifiers;

    fn bitor(self, rhs: Modifiers) -> Modifiers {
        self.union(rhs)
    }
}

/// The appearance of a cell: foreground, background, attributes and opacity.
///
/// [`opacity`](Style::opacity) controls how strongly the cell's colors are
/// composited over the canvas background. `255` is fully opaque; lower values
/// produce a frosted-glass / transparent look by alpha-blending toward the
/// canvas color (default black). Terminals cannot do true per-cell alpha, so
/// booms simulates crisp transparency by pre-blending each semi-transparent
/// cell into a solid ANSI/RGB color before writing it out.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Style {
    /// Foreground color.
    pub fg: Color,
    /// Background color.
    pub bg: Color,
    /// Text attributes.
    pub mods: Modifiers,
    /// Opacity in `0..=255`; `255` = fully opaque, `0` = fully transparent
    /// (renders as the canvas background color).
    pub opacity: u8,
}

impl Style {
    pub const DEFAULT: Style = Style {
        fg: Color::Default,
        bg: Color::Default,
        mods: Modifiers::NONE,
        opacity: 255,
    };

    pub const fn new() -> Style {
        Style::DEFAULT
    }

    pub const fn fg(mut self, color: Color) -> Style {
        self.fg = color;
        self
    }

    pub const fn bg(mut self, color: Color) -> Style {
        self.bg = color;
        self
    }

    pub const fn mods(mut self, mods: Modifiers) -> Style {
        self.mods = mods;
        self
    }

    /// Set the opacity (alpha) of this style, `0..=255`. Values below `255`
    /// make the cell semi-transparent by blending toward the canvas color.
    pub const fn opacity(mut self, opacity: u8) -> Style {
        self.opacity = opacity;
        self
    }

    /// True if every flag in `mods` is set in the style.
    pub const fn has(self, mods: Modifiers) -> bool {
        self.mods.contains(mods)
    }

    /// True if the style is fully transparent (renders as canvas color).
    pub const fn is_transparent(self) -> bool {
        self.opacity == 0
    }

    /// True if the style is semi-transparent.
    pub const fn is_semitransparent(self) -> bool {
        self.opacity > 0 && self.opacity < 255
    }
}

/// Alpha-composite `src` over `dst` with the given `alpha` (`0..=255`).
///
/// `alpha = 255` returns `src`; `alpha = 0` returns `dst`. `Color::Default`
/// on either side is treated as black `(0, 0, 0)` for blending purposes.
pub const fn alpha_over(src: Color, dst: Color, alpha: u8) -> Color {
    let sr = src_channels(src).0;
    let sg = src_channels(src).1;
    let sb = src_channels(src).2;
    let dr = src_channels(dst).0;
    let dg = src_channels(dst).1;
    let db = src_channels(dst).2;
    let t = alpha as u32;
    let r = lerp_u8(dr, sr, t);
    let g = lerp_u8(dg, sg, t);
    let b = lerp_u8(db, sb, t);
    Color::Rgb(r, g, b)
}

/// Blend `src` toward `dst` by fraction `t/255` (per-channel linear interp).
const fn lerp_u8(dst: u8, src: u8, t: u32) -> u8 {
    ((dst as u32 * inv_u32(t) + src as u32 * t) / 255) as u8
}

const fn inv_u32(t: u32) -> u32 {
    255u32 - t
}

/// The RGB channels of a color; `Color::Default` maps to black `(0, 0, 0)`.
const fn src_channels(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Default => (0, 0, 0),
        Color::Ansi(n) => ansi_rgb(n),
        Color::Ansi256(n) => ansi256_rgb(n),
        Color::Rgb(r, g, b) => (r, g, b),
    }
}

/// The sRGB-ish value of one of the 16 ANSI colors.
const fn ansi_rgb(n: u8) -> (u8, u8, u8) {
    match n {
        0 => (0, 0, 0),        // black
        1 => (128, 0, 0),      // red
        2 => (0, 128, 0),      // green
        3 => (128, 128, 0),    // yellow
        4 => (0, 0, 128),      // blue
        5 => (128, 0, 128),    // magenta
        6 => (0, 128, 128),    // cyan
        7 => (192, 192, 192),  // white
        8 => (128, 128, 128),  // bright black
        9 => (255, 0, 0),      // bright red
        10 => (0, 255, 0),     // bright green
        11 => (255, 255, 0),   // bright yellow
        12 => (0, 0, 255),     // bright blue
        13 => (255, 0, 255),   // bright magenta
        14 => (0, 255, 255),   // bright cyan
        15 => (255, 255, 255), // bright white
        _ => (0, 0, 0),
    }
}

/// The RGB value of a 256-color index. Indices 0..=15 are the ANSI colors;
/// 16..=231 are the 6x6x6 color cube; 232..=255 are the grayscale ramp.
const fn ansi256_rgb(n: u8) -> (u8, u8, u8) {
    if n <= 15 {
        return ansi_rgb(n);
    }
    if n <= 231 {
        let n = n - 16;
        let r = (n / 36) * 40 + 55;
        let g = ((n % 36) / 6) * 40 + 55;
        let b = (n % 6) * 40 + 55;
        return (r, g, b);
    }
    let n = n - 232;
    let v = n * 10 + 8;
    (v, v, v)
}

impl Color {
    /// Composite this color over `dst` with `alpha` (`0..=255`), returning a
    /// solid `Color::Rgb`. `alpha = 255` returns this color; `alpha = 0`
    /// returns `dst`.
    pub const fn alpha_over(self, dst: Color, alpha: u8) -> Color {
        alpha_over(self, dst, alpha)
    }
}
