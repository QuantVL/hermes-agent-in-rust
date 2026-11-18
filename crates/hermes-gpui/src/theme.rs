//! Theme tokens for the Hermes desktop shell.
//!
//! Tokens are plain Rust values (no GPUI types) so they can be constructed and
//! asserted in headless unit tests. Later UI tasks map these onto GPUI styles.

/// One 8-bit-per-channel RGBA colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// True when the colour is fully opaque (alpha == 255).
    pub const fn is_opaque(&self) -> bool {
        self.a == u8::MAX
    }
}

/// Semantic colours. Every token is fully opaque by default: a transparent
/// application background would leak the OS desktop through the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorTokens {
    pub bg: Rgba,
    pub fg: Rgba,
    pub surface: Rgba,
    pub border: Rgba,
    pub accent: Rgba,
    pub muted: Rgba,
}

impl Default for ColorTokens {
    fn default() -> Self {
        Self {
            bg: Rgba::new(0x1e, 0x1e, 0x1e, u8::MAX),
            fg: Rgba::new(0xea, 0xea, 0xea, u8::MAX),
            surface: Rgba::new(0x28, 0x28, 0x28, u8::MAX),
            border: Rgba::new(0x3a, 0x3a, 0x3a, u8::MAX),
            accent: Rgba::new(0x6a, 0x9e, 0xff, u8::MAX),
            muted: Rgba::new(0x9a, 0x9a, 0x9a, u8::MAX),
        }
    }
}

/// Font families and sizes used across the shell.
#[derive(Debug, Clone, PartialEq)]
pub struct FontTokens {
    pub ui_family: String,
    pub mono_family: String,
    pub ui_size: f32,
    pub mono_size: f32,
    pub line_height: f32,
}

impl Default for FontTokens {
    fn default() -> Self {
        Self {
            ui_family: "Inter".to_string(),
            mono_family: "JetBrains Mono".to_string(),
            ui_size: 14.0,
            mono_size: 13.0,
            line_height: 1.6,
        }
    }
}

/// Complete visual theme: colours plus typography.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Theme {
    pub colors: ColorTokens,
    pub fonts: FontTokens,
}
