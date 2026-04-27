use crate::common::insert_many_styles;
use crate::theme_structures::{Rgb, Style, Theme};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub enum StudioVariant {
    Aviel,
    Classic,
}

#[derive(Debug, Clone, Copy)]
pub struct StudioPalette {
    bg_base: Rgb,
    bg_dotted_statements: Rgb,
    bg_label: Rgb,
    bg_local_variable: Rgb,
    bg_macro: Rgb,
    bg_string: Rgb,
    text: Rgb,
    navy: Rgb,
    fg_comment_documentation: Rgb,
    fg_label: Rgb,
    fg_local_variable: Rgb,
    fg_macro: Rgb,
    fg_keyword_operator: Rgb,
    fg_preprocessor: Rgb,
    fg_string: Rgb,
    fg_type_builtin: Rgb,
    green: Rgb,
    purple: Rgb,
    olive: Rgb,
    gray: Rgb,
    red: Rgb,
    magenta: Rgb,
    maroon: Rgb,
    silver: Rgb,
}

impl StudioPalette {
    #[must_use]
    pub fn for_variant(variant: StudioVariant) -> Self {
        match variant {
            StudioVariant::Aviel => Self::aviel(),
            StudioVariant::Classic => Self::studio_default(),
        }
    }

    fn studio_default() -> Self {
        Self {
            bg_base: Rgb::new(0xff, 0xff, 0xff),
            bg_dotted_statements: Rgb::new(0xff, 0xff, 0xff),
            bg_label: Rgb::new(0xff, 0xff, 0xff),
            bg_local_variable: Rgb::new(0xff, 0xff, 0xff),
            bg_macro: Rgb::new(0xff, 0xff, 0xff),
            bg_string: Rgb::new(0xff, 0xff, 0xff),
            text: Rgb::new(0x00, 0x00, 0x00),
            navy: Rgb::new(0x00, 0x00, 0x80),
            fg_comment_documentation: Rgb::new(0x00, 0x80, 0x00),
            fg_label: Rgb::new(0x00, 0x00, 0xff),
            fg_local_variable: Rgb::new(0x00, 0x00, 0x80),
            fg_macro: Rgb::new(0xff, 0x00, 0x00),
            fg_keyword_operator: Rgb::new(0x80, 0x00, 0x00),
            fg_preprocessor: Rgb::new(0xff, 0x00, 0x00),
            fg_string: Rgb::new(0x00, 0x80, 0x00),
            fg_type_builtin: Rgb::new(0x80, 0x80, 0x00),
            green: Rgb::new(0x00, 0x80, 0x00),
            purple: Rgb::new(0x80, 0x00, 0x80),
            olive: Rgb::new(0x80, 0x80, 0x00),
            gray: Rgb::new(0x80, 0x80, 0x80),
            red: Rgb::new(0xff, 0x00, 0x00),
            magenta: Rgb::new(0xff, 0x00, 0xff),
            maroon: Rgb::new(0x80, 0x00, 0x00),
            silver: Rgb::new(0xc0, 0xc0, 0xc0),
        }
    }

    fn aviel() -> Self {
        Self {
            bg_base: Rgb::new(0xff, 0xff, 0xff),
            bg_dotted_statements: Rgb::new(0xc0, 0xc0, 0xc0),
            bg_label: Rgb::new(0xff, 0xff, 0x00),
            bg_local_variable: Rgb::new(0xdb, 0xff, 0xff),
            bg_macro: Rgb::new(0xc0, 0xc0, 0xc0),
            bg_string: Rgb::new(0xff, 0xc8, 0xff),
            text: Rgb::new(0x00, 0x00, 0x00),
            navy: Rgb::new(0x00, 0x00, 0x80),
            fg_comment_documentation: Rgb::new(0x00, 0x00, 0x80),
            fg_label: Rgb::new(0x80, 0x00, 0x00),
            fg_local_variable: Rgb::new(0x80, 0x00, 0x00),
            fg_macro: Rgb::new(0x00, 0x00, 0xff),
            fg_keyword_operator: Rgb::new(0x00, 0x00, 0x80),
            fg_preprocessor: Rgb::new(0x00, 0x00, 0xff),
            fg_string: Rgb::new(0x00, 0x00, 0x00),
            fg_type_builtin: Rgb::new(0x00, 0x80, 0x80),
            green: Rgb::new(0x00, 0x80, 0x00),
            purple: Rgb::new(0x80, 0x00, 0x80),
            olive: Rgb::new(0x80, 0x80, 0x00),
            gray: Rgb::new(0x80, 0x80, 0x80),
            red: Rgb::new(0xff, 0x00, 0x00),
            magenta: Rgb::new(0xff, 0x00, 0xff),
            maroon: Rgb::new(0x80, 0x00, 0x00),
            silver: Rgb::new(0xc0, 0xc0, 0xc0),
        }
    }
}

#[must_use]
pub fn build_studio_theme(variant: StudioVariant) -> Theme {
    let p = StudioPalette::for_variant(variant);

    let fg = |color: Rgb| Style {
        fg: Some(color),
        ..Style::default()
    };
    let bg = |color: Rgb| Style {
        bg: Some(color),
        ..Style::default()
    };
    let fg_bg = |front: Rgb, back: Rgb| Style {
        fg: Some(front),
        bg: Some(back),
        ..Style::default()
    };

    let mut styles = BTreeMap::new();
    let mut ui = BTreeMap::new();

    let normal_style = fg_bg(p.text, p.bg_base);
    let macro_style = fg_bg(p.fg_macro, p.bg_macro);
    let label_style = fg_bg(p.fg_label, p.bg_label);
    let dotted_statement_style = fg_bg(p.text, p.bg_dotted_statements);
    let string_style = fg_bg(p.fg_string, p.bg_string);
    let variable_style = fg_bg(p.fg_local_variable, p.bg_local_variable);

    let _ = styles.insert("normal".to_string(), normal_style);
    let _ = styles.insert("selection".to_string(), bg(p.silver));
    let _ = styles.insert("statusline".to_string(), fg_bg(p.text, p.bg_base));
    let _ = styles.insert("ignore".to_string(), fg(p.gray));
    let _ = styles.insert("warning".to_string(), fg(p.purple));
    let _ = styles.insert("error".to_string(), fg(p.text));
    insert_many_styles(
        &mut styles,
        &["comment", "variable.member.sql"],
        fg(p.green),
    );

    insert_many_styles(
        &mut styles,
        &[
            "constant.builtin",
            "variable.builtin",
            "keyword.directive",
            "variable.member.oref",
        ],
        fg(p.fg_preprocessor),
    );

    styles.insert("function.macro".to_string(), macro_style);
    styles.insert("keyword".to_string(), fg(p.red));
    insert_many_styles(
        &mut styles,
        &[
            "keyword.debug",
            "punctuation.special",
            "variable.member",
            "number",
        ],
        normal_style,
    );

    insert_many_styles(
        &mut styles,
        &["keyword.modifier", "string.regexp"],
        fg(p.olive),
    );

    styles.insert("keyword.type".to_string(), fg(p.navy));
    styles.insert(
        "comment.documentation".to_string(),
        fg(p.fg_comment_documentation),
    );
    styles.insert("keyword.operator".to_string(), fg(p.fg_keyword_operator));
    styles.insert("label".to_string(), label_style);

    insert_many_styles(
        &mut styles,
        &["punctuation.bracket", "type.definition"],
        fg(p.purple),
    );

    insert_many_styles(
        &mut styles,
        &["punctuation.bracket.json", "variable.parameter"],
        fg(p.magenta),
    );

    styles.insert(
        "punctuation.special.dots".to_string(),
        dotted_statement_style,
    );
    styles.insert("string".to_string(), string_style);
    styles.insert("type.builtin".to_string(), fg(p.fg_type_builtin));
    styles.insert("variable".to_string(), variable_style);
    insert_many_styles(
        &mut styles,
        &["comment.inactive", "markup.heading"],
        fg(p.gray),
    );
    // MOVING ON TO REGULAR CAPTURES (NOT OBJECTSCRIPT SPECIFIC)
    let _ = styles.insert("property".to_string(), fg(p.maroon));
    insert_many_styles(&mut styles, &["punctuation.delimiter", "tag"], fg(p.navy));
    // UI ROLES
    let _ = ui.insert("default_fg".to_string(), fg(p.text));
    let _ = ui.insert("default_bg".to_string(), bg(p.bg_base));
    let _ = ui.insert("statusline".to_string(), fg_bg(p.text, p.bg_base));
    let _ = ui.insert("statusline_inactive".to_string(), fg_bg(p.gray, p.bg_base));
    let _ = ui.insert("tab_active".to_string(), fg_bg(p.text, p.bg_base));
    let _ = ui.insert("tab_inactive".to_string(), fg_bg(p.gray, p.bg_base));
    let _ = ui.insert("selection".to_string(), bg(p.silver));
    let _ = ui.insert("cursorline".to_string(), bg(p.silver));

    Theme::from_parts(styles, ui)
}
