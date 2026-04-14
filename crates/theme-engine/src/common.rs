use crate::theme_structures::*;
use std::collections::BTreeMap;
/// Normalizes a theme capture name for lookup.
///
/// The normalization trims whitespace, removes an optional `@` prefix, and lowercases.

#[must_use]
pub fn normalize_capture_name(capture_name: &str) -> String {
    let trimmed = capture_name.trim();
    let without_prefix = trimmed.strip_prefix('@').unwrap_or(trimmed);
    without_prefix.to_ascii_lowercase()
}

/// Loads a built-in theme by name or alias.
///
/// # Errors
///
/// Returns an error for unknown theme names or malformed embedded theme data.
pub fn load_theme(name: &str) -> Result<Theme, ThemeError> {
    Theme::from_builtin_name(name)
}

pub fn insert_many_styles(styles: &mut BTreeMap<String, Style>, names: &[&str], style: Style) {
    for name in names {
        let _ = styles.insert((*name).to_string(), style);
    }
}
