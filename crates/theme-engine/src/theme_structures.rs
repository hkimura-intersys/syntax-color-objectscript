use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Theme {
    pub styles: BTreeMap<String, Style>,
    pub ui: BTreeMap<String, Style>,
}

pub const BUILTIN_THEME_NAMES: [&str; 14] = [
    "tokyonight-night",
    "tokyonight-storm",
    "tokyonight-moon",
    "tokyonight-day",
    "tokyonight-dark",
    "tokyonight-light",
    "catppuccin-latte",
    "catppuccin-frappe",
    "catppuccin-macchiato",
    "catppuccin-mocha",
    "aviel",
    "studio-default",
    "solarized-dark",
    "solarized-light",
];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BuiltinTheme {
    TokyoNightDark,
    TokyoNightNight,
    TokyoNightStorm,
    TokyoNightMoon,
    TokyoNightLight,
    TokyoNightDay,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    Aviel,
    StudioDefault,
    SolarizedDark,
    SolarizedLight,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct Style {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fg: Option<Rgb>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg: Option<Rgb>,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum UiRole {
    DefaultFg,
    DefaultBg,
    Statusline,
    StatuslineInactive,
    TabActive,
    TabInactive,
    Selection,
    Cursorline,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WrappedThemeDocument {
    #[serde(default)]
    pub(crate) styles: BTreeMap<String, Style>,
    #[serde(default)]
    pub(crate) ui: BTreeMap<String, Style>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ThemeDocument {
    Wrapped(WrappedThemeDocument),
    Flat(BTreeMap<String, Style>),
}

#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("failed to parse theme JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to parse theme TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("unknown style key '{0}' in palette template (must be one of BASE_THEME_STYLE_KEYS)")]
    UnknownTemplateStyleKey(String),
    #[error("unknown ui key '{0}' in palette template (must be one of BASE_THEME_UI_KEYS)")]
    UnknownTemplateUiKey(String),
    #[error("unknown color name '{color_name}' referenced by '{reference}'")]
    UnknownTemplateColor {
        color_name: String,
        reference: String,
    },
    #[error("duplicate color name '{0}' after case-insensitive normalization")]
    DuplicateTemplateColorName(String),
    #[error("empty color name is not allowed in palette template")]
    EmptyTemplateColorName,
    #[error(
        "unknown built-in theme '{0}', available: tokyonight-night, tokyonight-storm, tokyonight-moon, tokyonight-day, tokyonight-dark, tokyonight-light, catppuccin-latte, catppuccin-frappe, catppuccin-macchiato, catppuccin-mocha, aviel-reg, studio-classic-reg, solarized-dark, solarized-light"
    )]
    UnknownBuiltinTheme(String),
}

/// Returns canonical names of built-in themes.
#[must_use]
pub const fn available_themes() -> &'static [&'static str] {
    &BUILTIN_THEME_NAMES
}
