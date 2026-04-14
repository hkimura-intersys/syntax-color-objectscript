use crate::common::normalize_capture_name;
use crate::theme_structures::UiRole;

impl UiRole {
    /// Returns the canonical role key used in theme documents.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::DefaultFg => "default_fg",
            Self::DefaultBg => "default_bg",
            Self::Statusline => "statusline",
            Self::StatuslineInactive => "statusline_inactive",
            Self::TabActive => "tab_active",
            Self::TabInactive => "tab_inactive",
            Self::Selection => "selection",
            Self::Cursorline => "cursorline",
        }
    }

    /// Parses a UI role from a key or alias.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match normalize_capture_name(name).as_str() {
            "default_fg" | "defaultfg" | "terminal_fg" | "terminalfg" => Some(Self::DefaultFg),
            "default_bg" | "defaultbg" | "terminal_bg" | "terminalbg" => Some(Self::DefaultBg),
            "statusline" | "status_line" => Some(Self::Statusline),
            "statusline_inactive" | "status_line_inactive" | "statuslineinactive" => {
                Some(Self::StatuslineInactive)
            }
            "tab_active" | "tabactive" | "tab" => Some(Self::TabActive),
            "tab_inactive" | "tabinactive" => Some(Self::TabInactive),
            "selection" => Some(Self::Selection),
            "cursorline" | "cursor_line" => Some(Self::Cursorline),
            _ => None,
        }
    }
}
