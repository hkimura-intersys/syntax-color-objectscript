use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const BUILTIN_THEME_NAMES: [&str; 6] = [
    "tokyonight-dark",
    "tokyonight-moon",
    "tokyonight-light",
    "tokyonight-day",
    "solarized-dark",
    "solarized-light",
];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BuiltinTheme {
    TokyoNightDark,
    TokyoNightMoon,
    TokyoNightLight,
    TokyoNightDay,
    SolarizedDark,
    SolarizedLight,
}

impl BuiltinTheme {
    /// Returns the canonical name used in configuration and CLI arguments.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::TokyoNightDark => "tokyonight-dark",
            Self::TokyoNightMoon => "tokyonight-moon",
            Self::TokyoNightLight => "tokyonight-light",
            Self::TokyoNightDay => "tokyonight-day",
            Self::SolarizedDark => "solarized-dark",
            Self::SolarizedLight => "solarized-light",
        }
    }

    /// Parses a built-in theme name or alias.
    ///
    /// Accepted aliases include `"tokyo-night"`, `"tokyo-day"`, and
    /// `"tokyonight-moon"`.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "tokyonight-dark" | "tokyo-night" => Some(Self::TokyoNightDark),
            "tokyonight-moon" => Some(Self::TokyoNightMoon),
            "tokyonight-light" | "tokyo-day" => Some(Self::TokyoNightLight),
            "tokyonight-day" => Some(Self::TokyoNightDay),
            "solarized-dark" => Some(Self::SolarizedDark),
            "solarized-light" => Some(Self::SolarizedLight),
            _ => None,
        }
    }

    /// Returns the embedded JSON source for this built-in theme.
    const fn source(self) -> &'static str {
        match self {
            Self::TokyoNightDark => include_str!("../themes/tokyonight-dark.json"),
            Self::TokyoNightMoon => include_str!("../themes/tokyonight-moon.json"),
            Self::TokyoNightLight => include_str!("../themes/tokyonight-light.json"),
            Self::TokyoNightDay => include_str!("../themes/tokyonight-day.json"),
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
    /// Creates an RGB triplet.
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
    /// Creates an empty theme.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a theme from a style map after normalizing capture names.
    #[must_use]
    pub fn from_styles(styles: BTreeMap<String, Style>) -> Self {
        let mut theme = Self::new();
        for (name, style) in styles {
            let _ = theme.insert(name, style);
        }
        theme
    }

    /// Inserts or replaces a style for a capture name.
    ///
    /// Capture names are normalized (trimmed, lowercased, optional `@` removed).
    /// Returns the previously associated style, if any.
    pub fn insert(&mut self, capture_name: impl AsRef<str>, style: Style) -> Option<Style> {
        self.styles
            .insert(normalize_capture_name(capture_name.as_ref()), style)
    }

    /// Returns the internal normalized style map.
    #[must_use]
    pub fn styles(&self) -> &BTreeMap<String, Style> {
        &self.styles
    }

    /// Returns the exact style for a capture after normalization.
    #[must_use]
    pub fn get_exact(&self, capture_name: &str) -> Option<&Style> {
        self.styles.get(&normalize_capture_name(capture_name))
    }

    /// Resolves a style using dotted-name fallback and finally `normal`.
    ///
    /// For example, `comment.documentation` falls back to `comment` before
    /// attempting `normal`.
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

    /// Parses a theme from JSON.
    ///
    /// Both wrapped (`{ "styles": { ... } }`) and flat style documents are accepted.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON cannot be parsed.
    pub fn from_json_str(input: &str) -> Result<Self, ThemeError> {
        let parsed = serde_json::from_str::<ThemeDocument>(input)?;
        Ok(Self::from_styles(parsed.into_styles()))
    }

    /// Parses a theme from TOML.
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML cannot be parsed.
    pub fn from_toml_str(input: &str) -> Result<Self, ThemeError> {
        let parsed = toml::from_str::<ThemeDocument>(input)?;
        Ok(Self::from_styles(parsed.into_styles()))
    }

    /// Loads a built-in theme from embedded JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if embedded theme JSON fails to parse.
    pub fn from_builtin(theme: BuiltinTheme) -> Result<Self, ThemeError> {
        Self::from_json_str(theme.source())
    }

    /// Loads a built-in theme from a name or alias.
    ///
    /// # Errors
    ///
    /// Returns [`ThemeError::UnknownBuiltinTheme`] for unknown names.
    pub fn from_builtin_name(name: &str) -> Result<Self, ThemeError> {
        let theme = BuiltinTheme::from_name(name)
            .ok_or_else(|| ThemeError::UnknownBuiltinTheme(name.trim().to_string()))?;
        Self::from_builtin(theme)
    }
}

/// Returns canonical names of built-in themes.
#[must_use]
pub const fn available_themes() -> &'static [&'static str] {
    &BUILTIN_THEME_NAMES
}

/// Loads a built-in theme by name or alias.
///
/// # Errors
///
/// Returns an error for unknown theme names or malformed embedded theme data.
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
        "unknown built-in theme '{0}', available: tokyonight-dark, tokyonight-moon, tokyonight-light, tokyonight-day, solarized-dark, solarized-light"
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
    /// Converts a parsed document to its style map representation.
    fn into_styles(self) -> BTreeMap<String, Style> {
        match self {
            ThemeDocument::Wrapped { styles } => styles,
            ThemeDocument::Flat(styles) => styles,
        }
    }
}

/// Normalizes a theme capture name for lookup.
///
/// The normalization trims whitespace, removes an optional `@` prefix, and lowercases.
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
    /// Verifies capture name normalization behavior.
    fn normalizes_capture_names() {
        assert_eq!(normalize_capture_name("@Comment.Doc"), "comment.doc");
        assert_eq!(normalize_capture_name(" keyword "), "keyword");
    }

    #[test]
    /// Verifies dotted fallback and `normal` fallback resolution.
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
    /// Verifies wrapped JSON theme documents parse correctly.
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
    /// Verifies flat TOML theme documents parse correctly.
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
    /// Verifies all built-ins load and contain a `normal` style.
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
    /// Verifies built-in enum loading works for a known theme.
    fn loads_built_in_theme_by_enum() {
        let theme = Theme::from_builtin(BuiltinTheme::TokyoNightDark)
            .expect("failed to load tokyonight-dark");
        assert!(theme.resolve("keyword").is_some());
    }

    #[test]
    /// Verifies unknown built-in names return the expected error.
    fn rejects_unknown_built_in_theme_name() {
        let err = load_theme("unknown-theme").expect_err("expected unknown-theme to fail");
        assert!(matches!(err, ThemeError::UnknownBuiltinTheme(_)));
    }

    #[test]
    /// Verifies theme aliases are accepted.
    fn supports_theme_aliases() {
        assert!(load_theme("tokyo-night").is_ok());
        assert!(load_theme("tokyo-day").is_ok());
        assert!(load_theme("tokyonight-moon").is_ok());
        assert!(load_theme("tokyonight-day").is_ok());
    }

    #[test]
    /// Verifies moon/day variants are distinct built-ins, not aliases.
    fn loads_distinct_tokyonight_variants() {
        let moon = load_theme("tokyonight-moon").expect("failed to load moon");
        let dark = load_theme("tokyonight-dark").expect("failed to load dark");
        let day = load_theme("tokyonight-day").expect("failed to load day");
        let light = load_theme("tokyonight-light").expect("failed to load light");

        assert_ne!(moon, dark, "moon should differ from dark");
        assert_ne!(day, light, "day should differ from light");
    }
}
