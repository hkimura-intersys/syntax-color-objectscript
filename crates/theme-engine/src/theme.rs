use crate::catppuccin::{build_catppuccin_theme, CatppuccinVariant};
use crate::common::*;
use crate::studio::{build_studio_theme, StudioVariant};
use crate::theme_structures::*;
use crate::tokyonight::{build_tokyonight_theme, TokyoNightVariant};
use std::collections::BTreeMap;

impl Theme {
    /// Creates an empty theme.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a theme from a style map after normalizing capture names.
    #[must_use]
    pub fn from_styles(styles: BTreeMap<String, Style>) -> Self {
        Self::from_parts(styles, BTreeMap::new())
    }

    /// Creates a theme from syntax-style and UI-role maps after normalization.
    #[must_use]
    pub fn from_parts(styles: BTreeMap<String, Style>, ui: BTreeMap<String, Style>) -> Self {
        let mut theme = Self::new();
        for (name, style) in styles {
            let _ = theme.insert(name, style);
        }
        for (name, style) in ui {
            let _ = theme.insert_ui(name, style);
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

    /// Inserts or replaces a UI role style.
    ///
    /// Role names are normalized like capture names.
    /// Returns the previously associated style, if any.
    pub fn insert_ui(&mut self, role_name: impl AsRef<str>, style: Style) -> Option<Style> {
        self.ui
            .insert(normalize_capture_name(role_name.as_ref()), style)
    }

    /// Returns the internal normalized UI role map.
    #[must_use]
    pub fn ui_styles(&self) -> &BTreeMap<String, Style> {
        &self.ui
    }

    /// Returns the exact style for a capture after normalization.
    #[must_use]
    pub fn get_exact(&self, capture_name: &str) -> Option<&Style> {
        self.styles.get(&normalize_capture_name(capture_name))
    }

    /// Returns the exact UI role style after normalization.
    #[must_use]
    pub fn get_ui_exact(&self, role_name: &str) -> Option<&Style> {
        self.ui.get(&normalize_capture_name(role_name))
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

    /// Resolves a UI role from explicit UI map entries with compatibility fallbacks.
    ///
    /// This method first checks the dedicated `ui` map, then falls back to legacy
    /// entries in `styles` for compatibility with older themes.
    #[must_use]
    pub fn resolve_ui(&self, role_name: &str) -> Option<Style> {
        let normalized = normalize_capture_name(role_name);
        if let Some(style) = self.ui.get(&normalized).copied() {
            return Some(style);
        }
        if let Some(style) = self.styles.get(&normalized).copied() {
            return Some(style);
        }

        if let Some(role) = UiRole::from_name(&normalized) {
            return self.resolve_ui_role(role);
        }

        None
    }

    /// Resolves a typed UI role from explicit UI entries and fallbacks.
    #[must_use]
    pub fn resolve_ui_role(&self, role: UiRole) -> Option<Style> {
        let key = role.key();
        if let Some(style) = self.ui.get(key).copied() {
            return Some(style);
        }
        if let Some(style) = self.styles.get(key).copied() {
            return Some(style);
        }

        match role {
            UiRole::DefaultFg => self.styles.get("normal").and_then(|normal| {
                normal.fg.map(|fg| Style {
                    fg: Some(fg),
                    ..Style::default()
                })
            }),
            UiRole::DefaultBg => self.styles.get("normal").and_then(|normal| {
                normal.bg.map(|bg| Style {
                    bg: Some(bg),
                    ..Style::default()
                })
            }),
            UiRole::Statusline => self.styles.get("statusline").copied(),
            UiRole::StatuslineInactive => self
                .ui
                .get("statusline_inactive")
                .copied()
                .or_else(|| self.styles.get("statusline_inactive").copied())
                .or_else(|| self.styles.get("ignore").copied())
                .or_else(|| self.styles.get("statusline").copied()),
            UiRole::TabActive => self
                .ui
                .get("tab_active")
                .copied()
                .or_else(|| self.styles.get("tab_active").copied())
                .or_else(|| self.styles.get("statusline").copied()),
            UiRole::TabInactive => self
                .ui
                .get("tab_inactive")
                .copied()
                .or_else(|| self.styles.get("tab_inactive").copied())
                .or_else(|| self.styles.get("ignore").copied())
                .or_else(|| self.styles.get("statusline").copied()),
            UiRole::Selection => self
                .ui
                .get("selection")
                .copied()
                .or_else(|| self.styles.get("selection").copied()),
            UiRole::Cursorline => self
                .ui
                .get("cursorline")
                .copied()
                .or_else(|| self.styles.get("cursorline").copied())
                .or_else(|| self.styles.get("selection").copied()),
        }
    }

    /// Returns the theme default terminal foreground/background colors.
    ///
    /// Values are resolved from UI roles first (`default_fg`, `default_bg`), then
    /// from `styles.normal`.
    #[must_use]
    pub fn default_terminal_colors(&self) -> (Option<Rgb>, Option<Rgb>) {
        let fg = self
            .resolve_ui_role(UiRole::DefaultFg)
            .and_then(|style| style.fg)
            .or_else(|| self.styles.get("normal").and_then(|style| style.fg));
        let bg = self
            .resolve_ui_role(UiRole::DefaultBg)
            .and_then(|style| style.bg)
            .or_else(|| self.styles.get("normal").and_then(|style| style.bg));
        (fg, bg)
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
        Ok(parsed.into_theme())
    }

    /// Parses a theme from TOML.
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML cannot be parsed.
    pub fn from_toml_str(input: &str) -> Result<Self, ThemeError> {
        let parsed = toml::from_str::<ThemeDocument>(input)?;
        Ok(parsed.into_theme())
    }

    /// Loads a built-in theme from embedded JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if embedded theme JSON fails to parse.
    pub fn from_builtin(theme: BuiltinTheme) -> Result<Self, ThemeError> {
        match theme {
            BuiltinTheme::TokyoNightDark | BuiltinTheme::TokyoNightNight => {
                Ok(build_tokyonight_theme(TokyoNightVariant::Night))
            }
            BuiltinTheme::TokyoNightStorm => Ok(build_tokyonight_theme(TokyoNightVariant::Storm)),
            BuiltinTheme::TokyoNightMoon => Ok(build_tokyonight_theme(TokyoNightVariant::Moon)),
            BuiltinTheme::TokyoNightLight | BuiltinTheme::TokyoNightDay => {
                Ok(build_tokyonight_theme(TokyoNightVariant::Day))
            }
            BuiltinTheme::CatppuccinLatte => Ok(build_catppuccin_theme(CatppuccinVariant::Latte)),
            BuiltinTheme::CatppuccinFrappe => Ok(build_catppuccin_theme(CatppuccinVariant::Frappe)),
            BuiltinTheme::CatppuccinMacchiato => {
                Ok(build_catppuccin_theme(CatppuccinVariant::Macchiato))
            }
            BuiltinTheme::CatppuccinMocha => Ok(build_catppuccin_theme(CatppuccinVariant::Mocha)),
            BuiltinTheme::Aviel => Ok(build_studio_theme(StudioVariant::Aviel)),
            BuiltinTheme::StudioDefault => Ok(build_studio_theme(StudioVariant::Classic)),
            BuiltinTheme::SolarizedDark => {
                Self::from_json_str(include_str!("../themes/solarized-dark.json"))
            }
            BuiltinTheme::SolarizedLight => {
                Self::from_json_str(include_str!("../themes/solarized-light.json"))
            }
        }
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
