use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const BUILTIN_THEME_NAMES: [&str; 4] = [
    "tokyonight-dark",
    "tokyonight-light",
    "solarized-dark",
    "solarized-light",
];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BuiltinTheme {
    TokyoNightDark,
    TokyoNightLight,
    SolarizedDark,
    SolarizedLight,
}

impl BuiltinTheme {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::TokyoNightDark => "tokyonight-dark",
            Self::TokyoNightLight => "tokyonight-light",
            Self::SolarizedDark => "solarized-dark",
            Self::SolarizedLight => "solarized-light",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "tokyonight-dark" | "tokyonight-moon" | "tokyo-night" => Some(Self::TokyoNightDark),
            "tokyonight-light" | "tokyonight-day" | "tokyo-day" => Some(Self::TokyoNightLight),
            "solarized-dark" => Some(Self::SolarizedDark),
            "solarized-light" => Some(Self::SolarizedLight),
            _ => None,
        }
    }

    const fn source(self) -> &'static str {
        match self {
            Self::TokyoNightDark => include_str!("../themes/tokyonight-dark.json"),
            Self::TokyoNightLight => include_str!("../themes/tokyonight-light.json"),
            Self::SolarizedDark => include_str!("../themes/solarized-dark.json"),
            Self::SolarizedLight => include_str!("../themes/solarized-light.json"),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
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

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Theme {
    styles: BTreeMap<String, Style>,
}

impl Theme {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn from_styles(styles: BTreeMap<String, Style>) -> Self {
        let mut theme = Self::new();
        for (name, style) in styles {
            let _ = theme.insert(name, style);
        }
        theme
    }

    pub fn insert(&mut self, capture_name: impl AsRef<str>, style: Style) -> Option<Style> {
        self.styles
            .insert(normalize_capture_name(capture_name.as_ref()), style)
    }

    #[must_use]
    pub fn styles(&self) -> &BTreeMap<String, Style> {
        &self.styles
    }

    #[must_use]
    pub fn get_exact(&self, capture_name: &str) -> Option<&Style> {
        self.styles.get(&normalize_capture_name(capture_name))
    }

    #[must_use]
    pub fn resolve(&self, capture_name: &str) -> Option<&Style> {
        let mut key = normalize_capture_name(capture_name);

        loop {
            if let Some(style) = self.styles.get(&key) {
                return Some(style);
            }

            let Some(index) = key.rfind('.') else {
                break;
            };
            key.truncate(index);
        }

        self.styles.get("normal")
    }

    pub fn from_json_str(input: &str) -> Result<Self, ThemeError> {
        let parsed = serde_json::from_str::<ThemeDocument>(input)?;
        Ok(Self::from_styles(parsed.into_styles()))
    }

    pub fn from_toml_str(input: &str) -> Result<Self, ThemeError> {
        let parsed = toml::from_str::<ThemeDocument>(input)?;
        Ok(Self::from_styles(parsed.into_styles()))
    }

    pub fn from_builtin(theme: BuiltinTheme) -> Result<Self, ThemeError> {
        Self::from_json_str(theme.source())
    }

    pub fn from_builtin_name(name: &str) -> Result<Self, ThemeError> {
        let theme = BuiltinTheme::from_name(name)
            .ok_or_else(|| ThemeError::UnknownBuiltinTheme(name.trim().to_string()))?;
        Self::from_builtin(theme)
    }
}

#[must_use]
pub const fn available_themes() -> &'static [&'static str] {
    &BUILTIN_THEME_NAMES
}

pub fn load_theme(name: &str) -> Result<Theme, ThemeError> {
    Theme::from_builtin_name(name)
}

#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("failed to parse theme JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to parse theme TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error(
        "unknown built-in theme '{0}', available: tokyonight-dark, tokyonight-light, solarized-dark, solarized-light"
    )]
    UnknownBuiltinTheme(String),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ThemeDocument {
    Wrapped { styles: BTreeMap<String, Style> },
    Flat(BTreeMap<String, Style>),
}

impl ThemeDocument {
    fn into_styles(self) -> BTreeMap<String, Style> {
        match self {
            ThemeDocument::Wrapped { styles } => styles,
            ThemeDocument::Flat(styles) => styles,
        }
    }
}

#[must_use]
pub fn normalize_capture_name(capture_name: &str) -> String {
    let trimmed = capture_name.trim();
    let without_prefix = trimmed.strip_prefix('@').unwrap_or(trimmed);
    without_prefix.to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::{
        available_themes, load_theme, normalize_capture_name, BuiltinTheme, Rgb, Style, Theme,
        ThemeError,
    };

    #[test]
    fn normalizes_capture_names() {
        assert_eq!(normalize_capture_name("@Comment.Doc"), "comment.doc");
        assert_eq!(normalize_capture_name(" keyword "), "keyword");
    }

    #[test]
    fn resolves_dot_fallback_then_normal() {
        let mut theme = Theme::new();
        let _ = theme.insert(
            "comment",
            Style {
                fg: Some(Rgb::new(1, 2, 3)),
                ..Style::default()
            },
        );
        let _ = theme.insert(
            "normal",
            Style {
                fg: Some(Rgb::new(9, 9, 9)),
                ..Style::default()
            },
        );

        let comment = theme
            .resolve("@comment.documentation")
            .expect("missing comment");
        assert_eq!(comment.fg, Some(Rgb::new(1, 2, 3)));

        let unknown = theme.resolve("@does.not.exist").expect("missing normal");
        assert_eq!(unknown.fg, Some(Rgb::new(9, 9, 9)));
    }

    #[test]
    fn parses_json_theme_document() {
        let input = r#"
{
  "styles": {
    "@keyword": { "fg": { "r": 255, "g": 0, "b": 0 }, "bold": true },
    "normal": { "fg": { "r": 200, "g": 200, "b": 200 } }
  }
}
"#;

        let theme = Theme::from_json_str(input).expect("failed to parse json");
        let style = theme.resolve("keyword").expect("keyword style missing");
        assert_eq!(style.fg, Some(Rgb::new(255, 0, 0)));
        assert!(style.bold);
    }

    #[test]
    fn parses_toml_flat_theme_document() {
        let input = r#"
[normal]
fg = { r = 40, g = 41, b = 42 }

["@string"]
fg = { r = 120, g = 121, b = 122 }
italic = true
"#;

        let theme = Theme::from_toml_str(input).expect("failed to parse toml");
        let style = theme.resolve("string").expect("string style missing");
        assert_eq!(style.fg, Some(Rgb::new(120, 121, 122)));
        assert!(style.italic);
    }

    #[test]
    fn loads_all_built_in_themes() {
        for name in available_themes() {
            let theme = load_theme(name).expect("failed to load built-in theme");
            assert!(
                theme.get_exact("normal").is_some(),
                "missing normal style in {name}"
            );
        }
    }

    #[test]
    fn loads_built_in_theme_by_enum() {
        let theme = Theme::from_builtin(BuiltinTheme::TokyoNightDark)
            .expect("failed to load tokyonight-dark");
        assert!(theme.resolve("keyword").is_some());
    }

    #[test]
    fn rejects_unknown_built_in_theme_name() {
        let err = load_theme("unknown-theme").expect_err("expected unknown-theme to fail");
        assert!(matches!(err, ThemeError::UnknownBuiltinTheme(_)));
    }

    #[test]
    fn supports_theme_aliases() {
        assert!(load_theme("tokyo-night").is_ok());
        assert!(load_theme("tokyo-day").is_ok());
        assert!(load_theme("tokyonight-moon").is_ok());
    }
}
