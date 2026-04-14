pub mod built_in_theme;
pub mod catppuccin;
pub mod common;
pub mod rgb;
pub mod studio;
pub mod theme;
pub mod theme_document;
pub mod theme_structures;
pub mod tokyonight;
pub mod ui_role;
// #[cfg(test)]
// mod tests {
//     use super::{
//         available_themes, load_theme, normalize_capture_name, BuiltinTheme, Rgb, Style, Theme,
//         ThemeError, UiRole,
//     };

//     #[test]
//     /// Verifies capture name normalization behavior.
//     fn normalizes_capture_names() {
//         assert_eq!(normalize_capture_name("@Comment.Doc"), "comment.doc");
//         assert_eq!(normalize_capture_name(" keyword "), "keyword");
//     }

//     #[test]
//     /// Verifies dotted fallback and `normal` fallback resolution.
//     fn resolves_dot_fallback_then_normal() {
//         let mut theme = Theme::new();
//         let _ = theme.insert(
//             "comment",
//             Style {
//                 fg: Some(Rgb::new(1, 2, 3)),
//                 ..Style::default()
//             },
//         );
//         let _ = theme.insert(
//             "normal",
//             Style {
//                 fg: Some(Rgb::new(9, 9, 9)),
//                 ..Style::default()
//             },
//         );

//         let comment = theme
//             .resolve("@comment.documentation")
//             .expect("missing comment");
//         assert_eq!(comment.fg, Some(Rgb::new(1, 2, 3)));

//         let unknown = theme.resolve("@does.not.exist").expect("missing normal");
//         assert_eq!(unknown.fg, Some(Rgb::new(9, 9, 9)));
//     }

//     #[test]
//     /// Verifies wrapped JSON theme documents parse correctly.
//     fn parses_json_theme_document() {
//         let input = r#"
// {
//   "styles": {
//     "@keyword": { "fg": { "r": 255, "g": 0, "b": 0 }, "bold": true },
//     "normal": { "fg": { "r": 200, "g": 200, "b": 200 } }
//   }
// }
// "#;

//         let theme = Theme::from_json_str(input).expect("failed to parse json");
//         let style = theme.resolve("keyword").expect("keyword style missing");
//         assert_eq!(style.fg, Some(Rgb::new(255, 0, 0)));
//         assert!(style.bold);
//     }

//     #[test]
//     /// Verifies flat TOML theme documents parse correctly.
//     fn parses_toml_flat_theme_document() {
//         let input = r#"
// [normal]
// fg = { r = 40, g = 41, b = 42 }

// ["@string"]
// fg = { r = 120, g = 121, b = 122 }
// italic = true
// "#;

//         let theme = Theme::from_toml_str(input).expect("failed to parse toml");
//         let style = theme.resolve("string").expect("string style missing");
//         assert_eq!(style.fg, Some(Rgb::new(120, 121, 122)));
//         assert!(style.italic);
//     }

//     #[test]
//     /// Verifies all built-ins load and contain a `normal` style.
//     fn loads_all_built_in_themes() {
//         for name in available_themes() {
//             let theme = load_theme(name).expect("failed to load built-in theme");
//             assert!(
//                 theme.get_exact("normal").is_some(),
//                 "missing normal style in {name}"
//             );
//         }
//     }

//     #[test]
//     /// Verifies built-in enum loading works for a known theme.
//     fn loads_built_in_theme_by_enum() {
//         let theme = Theme::from_builtin(BuiltinTheme::TokyoNightDark)
//             .expect("failed to load tokyonight-dark");
//         assert!(theme.resolve("keyword").is_some());
//     }

//     #[test]
//     /// Verifies unknown built-in names return the expected error.
//     fn rejects_unknown_built_in_theme_name() {
//         let err = load_theme("unknown-theme").expect_err("expected unknown-theme to fail");
//         assert!(matches!(err, ThemeError::UnknownBuiltinTheme(_)));
//     }

//     #[test]
//     /// Verifies theme aliases are accepted.
//     fn supports_theme_aliases() {
//         assert!(load_theme("tokyo-night").is_ok());
//         assert!(load_theme("tokyo-day").is_ok());
//         assert!(load_theme("tokyonight-moon").is_ok());
//         assert!(load_theme("tokyonight-day").is_ok());
//     }

//     #[test]
//     /// Verifies moon/day variants are distinct built-ins, not aliases.
//     fn loads_distinct_tokyonight_variants() {
//         let moon = load_theme("tokyonight-moon").expect("failed to load moon");
//         let dark = load_theme("tokyonight-dark").expect("failed to load dark");
//         let day = load_theme("tokyonight-day").expect("failed to load day");
//         let light = load_theme("tokyonight-light").expect("failed to load light");

//         assert_ne!(moon, dark, "moon should differ from dark");
//         assert_ne!(day, light, "day should differ from light");
//     }

//     #[test]
//     /// Verifies built-in themes expose XML-relevant capture styles.
//     fn builtins_include_xml_capture_styles() {
//         for name in available_themes() {
//             let theme = load_theme(name).expect("failed to load built-in theme");
//             assert!(
//                 theme.get_exact("tag").is_some(),
//                 "missing XML tag style in {name}"
//             );
//             assert!(
//                 theme.get_exact("property").is_some(),
//                 "missing XML property style in {name}"
//             );
//         }
//     }

//     #[test]
//     /// Verifies wrapped documents can carry dedicated UI-role styles.
//     fn parses_ui_roles_from_wrapped_document() {
//         let input = r#"
// {
//   "styles": {
//     "normal": { "fg": { "r": 10, "g": 11, "b": 12 }, "bg": { "r": 13, "g": 14, "b": 15 } }
//   },
//   "ui": {
//     "default_fg": { "fg": { "r": 1, "g": 2, "b": 3 } },
//     "tab_active": { "fg": { "r": 20, "g": 21, "b": 22 }, "bg": { "r": 30, "g": 31, "b": 32 } }
//   }
// }
// "#;

//         let theme = Theme::from_json_str(input).expect("failed to parse json");
//         let default_fg = theme
//             .resolve_ui_role(UiRole::DefaultFg)
//             .expect("missing default fg");
//         assert_eq!(default_fg.fg, Some(Rgb::new(1, 2, 3)));

//         let tab = theme
//             .resolve_ui("tab_active")
//             .expect("missing tab_active role");
//         assert_eq!(tab.bg, Some(Rgb::new(30, 31, 32)));
//     }

//     #[test]
//     /// Verifies default terminal colors fall back to `normal` when UI roles are absent.
//     fn default_terminal_colors_fallback_to_normal() {
//         let theme = Theme::from_json_str(
//             r#"{
//   "styles": {
//     "normal": { "fg": { "r": 100, "g": 101, "b": 102 }, "bg": { "r": 110, "g": 111, "b": 112 } }
//   }
// }"#,
//         )
//         .expect("failed to parse json");

//         let (fg, bg) = theme.default_terminal_colors();
//         assert_eq!(fg, Some(Rgb::new(100, 101, 102)));
//         assert_eq!(bg, Some(Rgb::new(110, 111, 112)));
//     }

//     #[test]
//     /// Verifies UI role compatibility fallback uses legacy `styles` keys.
//     fn ui_role_falls_back_to_legacy_style_keys() {
//         let mut theme = Theme::new();
//         let _ = theme.insert(
//             "statusline",
//             Style {
//                 fg: Some(Rgb::new(1, 1, 1)),
//                 bg: Some(Rgb::new(2, 2, 2)),
//                 ..Style::default()
//             },
//         );
//         let _ = theme.insert(
//             "ignore",
//             Style {
//                 fg: Some(Rgb::new(3, 3, 3)),
//                 bg: Some(Rgb::new(4, 4, 4)),
//                 ..Style::default()
//             },
//         );

//         let active = theme
//             .resolve_ui_role(UiRole::TabActive)
//             .expect("missing active tab");
//         assert_eq!(active.bg, Some(Rgb::new(2, 2, 2)));

//         let inactive = theme
//             .resolve_ui_role(UiRole::TabInactive)
//             .expect("missing inactive tab");
//         assert_eq!(inactive.bg, Some(Rgb::new(4, 4, 4)));
//     }
// }
