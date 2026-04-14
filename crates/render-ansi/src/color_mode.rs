use crate::ansi_structures::ColorMode;
impl ColorMode {
    /// Parses a color mode from user input.
    ///
    #[must_use]
    pub fn from_name(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "truecolor" | "rgb" => Some(Self::TrueColor),
            "ansi256" => Some(Self::Ansi256),
            "ansi16" => Some(Self::Ansi16),
            _ => None,
        }
    }
}
