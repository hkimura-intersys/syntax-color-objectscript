use crate::theme_structures::*;

impl ThemeDocument {
    /// Converts a parsed document to a normalized [`Theme`].
    pub fn into_theme(self) -> Theme {
        match self {
            ThemeDocument::Wrapped(doc) => Theme::from_parts(doc.styles, doc.ui),
            ThemeDocument::Flat(styles) => Theme::from_styles(styles),
        }
    }
}
