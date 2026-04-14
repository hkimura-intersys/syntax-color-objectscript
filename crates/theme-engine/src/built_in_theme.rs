use crate::theme_structures::*;
impl BuiltinTheme {
    /// Parses a built-in theme name or alias.
    ///
    /// Accepted aliases include legacy names (`tokyo-night`, `tokyo-day`) and
    /// `tokyonight` (defaults to `tokyonight-moon`, matching upstream default style).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "tokyonight-dark" | "tokyo-night" => Some(Self::TokyoNightDark),
            "tokyonight-night" => Some(Self::TokyoNightNight),
            "tokyonight-storm" => Some(Self::TokyoNightStorm),
            "tokyonight-moon" | "tokyonight" => Some(Self::TokyoNightMoon),
            "tokyonight-light" | "tokyo-day" => Some(Self::TokyoNightLight),
            "tokyonight-day" => Some(Self::TokyoNightDay),
            "catppuccin-latte" => Some(Self::CatppuccinLatte),
            "catppuccin-frappe" => Some(Self::CatppuccinFrappe),
            "catppuccin-macchiato" => Some(Self::CatppuccinMacchiato),
            "catppuccin-mocha" => Some(Self::CatppuccinMocha),
            "aviel" | "studio-aviel" | "aviel-studio" => Some(Self::Aviel),
            "studio-default" | "studio-classic" => Some(Self::StudioDefault),
            "solarized-dark" => Some(Self::SolarizedDark),
            "solarized-light" => Some(Self::SolarizedLight),
            _ => None,
        }
    }
}
