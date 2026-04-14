use crate::theme_structures::Rgb;
impl Rgb {
    /// Creates an RGB triplet.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Blends `foreground` over `background` using Lua-style channel math.
    ///
    /// This mirrors the upstream Lua behavior:
    /// - alpha is not clamped before blending
    /// - each channel is clamped to `[0, 255]`
    /// - rounding is `floor(value + 0.5)`
    #[must_use]
    pub fn blend(foreground: Self, alpha: f64, background: Self) -> Self {
        let blend_channel = |fg: u8, bg: u8| -> u8 {
            let value = alpha * f64::from(fg) + (1.0 - alpha) * f64::from(bg);
            (value.clamp(0.0, 255.0) + 0.5).floor() as u8
        };

        Self {
            r: blend_channel(foreground.r, background.r),
            g: blend_channel(foreground.g, background.g),
            b: blend_channel(foreground.b, background.b),
        }
    }
}
