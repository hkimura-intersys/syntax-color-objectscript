use crate::common::insert_many_styles;
use crate::theme_structures::{Rgb, Style, Theme};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy)]
pub enum TokyoNightVariant {
    Night,
    Storm,
    Moon,
    Day,
}

#[derive(Debug, Clone, Copy)]
pub struct TokyoNightPalette {
    bg: Rgb,
    bg_highlight: Rgb,
    bg_statusline: Rgb,
    bg_visual: Rgb,
    black: Rgb,
    blue: Rgb,
    blue1: Rgb,
    blue5: Rgb,
    blue6: Rgb,
    comment: Rgb,
    cyan: Rgb,
    dark3: Rgb,
    dark5: Rgb,
    diff_add: Rgb,
    diff_change: Rgb,
    diff_delete: Rgb,
    error: Rgb,
    fg: Rgb,
    fg_dark: Rgb,
    fg_gutter: Rgb,
    fg_sidebar: Rgb,
    green: Rgb,
    green1: Rgb,
    hint: Rgb,
    info: Rgb,
    magenta: Rgb,
    orange: Rgb,
    purple: Rgb,
    red: Rgb,
    teal: Rgb,
    todo: Rgb,
    warning: Rgb,
    yellow: Rgb,
    terminal_black: Rgb,
}

impl TokyoNightPalette {
    #[must_use]
    pub fn for_variant(variant: TokyoNightVariant) -> Self {
        match variant {
            TokyoNightVariant::Night => Self::night(),
            TokyoNightVariant::Storm => Self::storm(),
            TokyoNightVariant::Moon => Self::moon(),
            TokyoNightVariant::Day => Self::day(),
        }
    }

    #[must_use]
    pub const fn night() -> Self {
        Self {
            bg: Rgb::new(0x1a, 0x1b, 0x26),
            bg_highlight: Rgb::new(0x29, 0x2e, 0x42),
            bg_statusline: Rgb::new(0x16, 0x16, 0x1e),
            bg_visual: Rgb::new(0x28, 0x34, 0x57),
            black: Rgb::new(0x15, 0x16, 0x1e),
            blue: Rgb::new(0x7a, 0xa2, 0xf7),
            blue1: Rgb::new(0x2a, 0xc3, 0xde),
            blue5: Rgb::new(0x89, 0xdd, 0xff),
            blue6: Rgb::new(0xb4, 0xf9, 0xf8),
            comment: Rgb::new(0x56, 0x5f, 0x89),
            cyan: Rgb::new(0x7d, 0xcf, 0xff),
            dark3: Rgb::new(0x54, 0x5c, 0x7e),
            dark5: Rgb::new(0x73, 0x7a, 0xa2),
            diff_add: Rgb::new(0x24, 0x3e, 0x4a),
            diff_change: Rgb::new(0x1f, 0x22, 0x31),
            diff_delete: Rgb::new(0x4a, 0x27, 0x2f),
            error: Rgb::new(0xdb, 0x4b, 0x4b),
            fg: Rgb::new(0xc0, 0xca, 0xf5),
            fg_dark: Rgb::new(0xa9, 0xb1, 0xd6),
            fg_gutter: Rgb::new(0x3b, 0x42, 0x61),
            fg_sidebar: Rgb::new(0xa9, 0xb1, 0xd6),
            green: Rgb::new(0x9e, 0xce, 0x6a),
            green1: Rgb::new(0x73, 0xda, 0xca),
            hint: Rgb::new(0x1a, 0xbc, 0x9c),
            info: Rgb::new(0x0d, 0xb9, 0xd7),
            magenta: Rgb::new(0xbb, 0x9a, 0xf7),
            orange: Rgb::new(0xff, 0x9e, 0x64),
            purple: Rgb::new(0x9d, 0x7c, 0xd8),
            red: Rgb::new(0xf7, 0x76, 0x8e),
            teal: Rgb::new(0x1a, 0xbc, 0x9c),
            todo: Rgb::new(0x7a, 0xa2, 0xf7),
            warning: Rgb::new(0xe0, 0xaf, 0x68),
            yellow: Rgb::new(0xe0, 0xaf, 0x68),
            terminal_black: Rgb::new(0x41, 0x48, 0x68),
        }
    }

    #[must_use]
    pub const fn storm() -> Self {
        Self {
            bg: Rgb::new(0x24, 0x28, 0x3b),
            bg_highlight: Rgb::new(0x29, 0x2e, 0x42),
            bg_statusline: Rgb::new(0x1f, 0x23, 0x35),
            bg_visual: Rgb::new(0x2e, 0x3c, 0x64),
            black: Rgb::new(0x1d, 0x20, 0x2f),
            blue: Rgb::new(0x7a, 0xa2, 0xf7),
            blue1: Rgb::new(0x2a, 0xc3, 0xde),
            blue5: Rgb::new(0x89, 0xdd, 0xff),
            blue6: Rgb::new(0xb4, 0xf9, 0xf8),
            comment: Rgb::new(0x56, 0x5f, 0x89),
            cyan: Rgb::new(0x7d, 0xcf, 0xff),
            dark3: Rgb::new(0x54, 0x5c, 0x7e),
            dark5: Rgb::new(0x73, 0x7a, 0xa2),
            diff_add: Rgb::new(0x2b, 0x48, 0x5a),
            diff_change: Rgb::new(0x27, 0x2d, 0x43),
            diff_delete: Rgb::new(0x52, 0x31, 0x3f),
            error: Rgb::new(0xdb, 0x4b, 0x4b),
            fg: Rgb::new(0xc0, 0xca, 0xf5),
            fg_dark: Rgb::new(0xa9, 0xb1, 0xd6),
            fg_gutter: Rgb::new(0x3b, 0x42, 0x61),
            fg_sidebar: Rgb::new(0xa9, 0xb1, 0xd6),
            green: Rgb::new(0x9e, 0xce, 0x6a),
            green1: Rgb::new(0x73, 0xda, 0xca),
            hint: Rgb::new(0x1a, 0xbc, 0x9c),
            info: Rgb::new(0x0d, 0xb9, 0xd7),
            magenta: Rgb::new(0xbb, 0x9a, 0xf7),
            orange: Rgb::new(0xff, 0x9e, 0x64),
            purple: Rgb::new(0x9d, 0x7c, 0xd8),
            red: Rgb::new(0xf7, 0x76, 0x8e),
            teal: Rgb::new(0x1a, 0xbc, 0x9c),
            todo: Rgb::new(0x7a, 0xa2, 0xf7),
            warning: Rgb::new(0xe0, 0xaf, 0x68),
            yellow: Rgb::new(0xe0, 0xaf, 0x68),
            terminal_black: Rgb::new(0x41, 0x48, 0x68),
        }
    }

    #[must_use]
    pub const fn moon() -> Self {
        Self {
            bg: Rgb::new(0x22, 0x24, 0x36),
            bg_highlight: Rgb::new(0x2f, 0x33, 0x4d),
            bg_statusline: Rgb::new(0x1e, 0x20, 0x30),
            bg_visual: Rgb::new(0x2d, 0x3f, 0x76),
            black: Rgb::new(0x1b, 0x1d, 0x2b),
            blue: Rgb::new(0x82, 0xaa, 0xff),
            blue1: Rgb::new(0x65, 0xbc, 0xff),
            blue5: Rgb::new(0x89, 0xdd, 0xff),
            blue6: Rgb::new(0xb4, 0xf9, 0xf8),
            comment: Rgb::new(0x63, 0x6d, 0xa6),
            cyan: Rgb::new(0x86, 0xe1, 0xfc),
            dark3: Rgb::new(0x54, 0x5c, 0x7e),
            dark5: Rgb::new(0x73, 0x7a, 0xa2),
            diff_add: Rgb::new(0x2a, 0x45, 0x56),
            diff_change: Rgb::new(0x25, 0x2a, 0x3f),
            diff_delete: Rgb::new(0x4b, 0x2a, 0x3d),
            error: Rgb::new(0xc5, 0x3b, 0x53),
            fg: Rgb::new(0xc8, 0xd3, 0xf5),
            fg_dark: Rgb::new(0x82, 0x8b, 0xb8),
            fg_gutter: Rgb::new(0x3b, 0x42, 0x61),
            fg_sidebar: Rgb::new(0x82, 0x8b, 0xb8),
            green: Rgb::new(0xc3, 0xe8, 0x8d),
            green1: Rgb::new(0x4f, 0xd6, 0xbe),
            hint: Rgb::new(0x4f, 0xd6, 0xbe),
            info: Rgb::new(0x0d, 0xb9, 0xd7),
            magenta: Rgb::new(0xc0, 0x99, 0xff),
            orange: Rgb::new(0xff, 0x96, 0x6c),
            purple: Rgb::new(0xfc, 0xa7, 0xea),
            red: Rgb::new(0xff, 0x75, 0x7f),
            teal: Rgb::new(0x4f, 0xd6, 0xbe),
            todo: Rgb::new(0x82, 0xaa, 0xff),
            warning: Rgb::new(0xff, 0xc7, 0x77),
            yellow: Rgb::new(0xff, 0xc7, 0x77),
            terminal_black: Rgb::new(0x44, 0x4a, 0x73),
        }
    }

    #[must_use]
    pub const fn day() -> Self {
        Self {
            bg: Rgb::new(0xe1, 0xe2, 0xe7),
            bg_highlight: Rgb::new(0xc4, 0xc8, 0xda),
            bg_statusline: Rgb::new(0xd0, 0xd5, 0xe3),
            bg_visual: Rgb::new(0xb7, 0xc1, 0xe3),
            black: Rgb::new(0xb4, 0xb5, 0xb9),
            blue: Rgb::new(0x2e, 0x7d, 0xe9),
            blue1: Rgb::new(0x18, 0x80, 0x92),
            blue5: Rgb::new(0x00, 0x6a, 0x83),
            blue6: Rgb::new(0x2e, 0x58, 0x57),
            comment: Rgb::new(0x84, 0x8c, 0xb5),
            cyan: Rgb::new(0x00, 0x71, 0x97),
            dark3: Rgb::new(0x89, 0x90, 0xb3),
            dark5: Rgb::new(0x68, 0x70, 0x9a),
            diff_add: Rgb::new(0xb7, 0xce, 0xd5),
            diff_change: Rgb::new(0xd5, 0xd9, 0xe4),
            diff_delete: Rgb::new(0xda, 0xba, 0xbe),
            error: Rgb::new(0xc6, 0x43, 0x43),
            fg: Rgb::new(0x37, 0x60, 0xbf),
            fg_dark: Rgb::new(0x61, 0x72, 0xb0),
            fg_gutter: Rgb::new(0xa8, 0xae, 0xcb),
            fg_sidebar: Rgb::new(0x61, 0x72, 0xb0),
            green: Rgb::new(0x58, 0x75, 0x39),
            green1: Rgb::new(0x38, 0x70, 0x68),
            hint: Rgb::new(0x11, 0x8c, 0x74),
            info: Rgb::new(0x07, 0x87, 0x9d),
            magenta: Rgb::new(0x98, 0x54, 0xf1),
            orange: Rgb::new(0xb1, 0x5c, 0x00),
            purple: Rgb::new(0x78, 0x47, 0xbd),
            red: Rgb::new(0xf5, 0x2a, 0x65),
            teal: Rgb::new(0x11, 0x8c, 0x74),
            todo: Rgb::new(0x2e, 0x7d, 0xe9),
            warning: Rgb::new(0x8c, 0x6c, 0x3e),
            yellow: Rgb::new(0x8c, 0x6c, 0x3e),
            terminal_black: Rgb::new(0xa1, 0xa6, 0xc5),
        }
    }
}

#[must_use]
pub fn build_tokyonight_theme(variant: TokyoNightVariant) -> Theme {
    let p = TokyoNightPalette::for_variant(variant);

    let fg = |color: Rgb| Style {
        fg: Some(color),
        ..Style::default()
    };
    let fg_italic = |color: Rgb| Style {
        fg: Some(color),
        italic: true,
        ..Style::default()
    };
    let fg_bold = |color: Rgb| Style {
        fg: Some(color),
        bold: true,
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

    let keyword_style = fg_italic(p.purple);
    let type_builtin = fg(Rgb::blend(p.blue1, 0.8, p.bg));
    let parameter_builtin = fg(Rgb::blend(p.yellow, 0.8, p.fg));
    let tag_delimiter_tsx = fg(Rgb::blend(p.blue, 0.7, p.bg));

    let _ = styles.insert("normal".to_string(), fg_bg(p.fg, p.bg));
    let _ = styles.insert("selection".to_string(), bg(p.bg_visual));
    let _ = styles.insert(
        "statusline".to_string(),
        fg_bg(p.fg_sidebar, p.bg_statusline),
    );
    let _ = styles.insert("ignore".to_string(), fg(p.dark3));
    let _ = styles.insert("warning".to_string(), fg_bold(p.warning));
    let _ = styles.insert("error".to_string(), fg_bold(p.error));

    insert_many_styles(&mut styles, &["annotation", "attribute"], fg(p.cyan));
    let _ = styles.insert("attribute.builtin".to_string(), fg(p.blue1));
    let _ = styles.insert("boolean".to_string(), fg(p.orange));
    let _ = styles.insert("character".to_string(), fg(p.green));
    insert_many_styles(
        &mut styles,
        &["character.printf", "character.special"],
        fg(p.blue1),
    );

    let comment_style = fg_italic(p.comment);
    insert_many_styles(
        &mut styles,
        &["comment", "comment.documentation", "comment.note"],
        comment_style,
    );
    let _ = styles.insert("comment.error".to_string(), fg(p.error));
    let _ = styles.insert("comment.hint".to_string(), fg(p.hint));
    let _ = styles.insert("comment.info".to_string(), fg(p.info));
    let _ = styles.insert("comment.todo".to_string(), fg(p.todo));
    let _ = styles.insert("comment.warning".to_string(), fg(p.warning));

    let _ = styles.insert("conceal".to_string(), fg(p.dark5));
    let _ = styles.insert("constant".to_string(), fg(p.orange));
    let _ = styles.insert("constant.builtin".to_string(), fg(p.blue1));
    let _ = styles.insert("constant.macro".to_string(), fg(p.cyan));

    let _ = styles.insert("constructor".to_string(), fg(p.magenta));
    let _ = styles.insert("constructor.tsx".to_string(), fg(p.blue1));

    let _ = styles.insert("diff.delta".to_string(), bg(p.diff_change));
    let _ = styles.insert("diff.minus".to_string(), bg(p.diff_delete));
    let _ = styles.insert("diff.plus".to_string(), bg(p.diff_add));

    let _ = styles.insert("embedded".to_string(), fg(p.green));

    insert_many_styles(
        &mut styles,
        &[
            "function",
            "function.call",
            "function.method",
            "function.method.call",
        ],
        fg(p.blue),
    );
    let _ = styles.insert("function.builtin".to_string(), fg(p.blue1));
    let _ = styles.insert("function.macro".to_string(), fg(p.cyan));

    insert_many_styles(
        &mut styles,
        &[
            "keyword",
            "keyword.coroutine",
            "keyword.return",
            "keyword.type",
            "keyword.modifier",
            "type.qualifier",
        ],
        keyword_style,
    );
    insert_many_styles(
        &mut styles,
        &[
            "keyword.conditional",
            "keyword.conditional.ternary",
            "keyword.exception",
            "keyword.function",
            "keyword.repeat",
        ],
        fg(p.magenta),
    );
    let _ = styles.insert("keyword.debug".to_string(), fg(p.orange));
    insert_many_styles(
        &mut styles,
        &[
            "keyword.directive",
            "keyword.directive.define",
            "keyword.import",
            "module",
        ],
        fg(p.cyan),
    );
    let _ = styles.insert("keyword.operator".to_string(), fg(p.blue5));
    let _ = styles.insert("keyword.storage".to_string(), fg(p.blue1));

    let _ = styles.insert("label".to_string(), fg(p.blue));

    let _ = styles.insert("markup".to_string(), Style::default());
    let _ = styles.insert("markup.heading".to_string(), fg_bold(p.blue));
    let heading_rainbow = [
        p.blue, p.yellow, p.green, p.teal, p.magenta, p.purple, p.orange, p.red,
    ];
    for (index, color) in heading_rainbow.iter().enumerate() {
        let _ = styles.insert(
            format!("markup.heading.{}", index + 1),
            Style {
                fg: Some(*color),
                bg: Some(Rgb::blend(*color, 0.1, p.bg)),
                bold: true,
                ..Style::default()
            },
        );
    }
    let _ = styles.insert(
        "markup.italic".to_string(),
        Style {
            italic: true,
            ..Style::default()
        },
    );
    let _ = styles.insert("markup.link".to_string(), fg(p.teal));
    insert_many_styles(
        &mut styles,
        &["markup.link.label", "markup.link.label.symbol"],
        fg(p.blue1),
    );
    let _ = styles.insert(
        "markup.link.url".to_string(),
        Style {
            fg: Some(p.teal),
            underline: true,
            ..Style::default()
        },
    );
    let _ = styles.insert("markup.list".to_string(), fg(p.blue5));
    let _ = styles.insert("markup.list.checked".to_string(), fg(p.green1));
    let _ = styles.insert("markup.list.markdown".to_string(), fg_bold(p.orange));
    let _ = styles.insert("markup.list.unchecked".to_string(), fg(p.blue));
    let _ = styles.insert("markup.math".to_string(), fg(p.blue1));
    let _ = styles.insert("markup.quote".to_string(), comment_style);
    insert_many_styles(
        &mut styles,
        &["markup.raw", "markup.raw.block"],
        fg(p.green),
    );
    let _ = styles.insert(
        "markup.raw.markdown_inline".to_string(),
        fg_bg(p.blue, p.terminal_black),
    );
    let _ = styles.insert("markup.strikethrough".to_string(), fg(p.comment));
    let _ = styles.insert(
        "markup.strong".to_string(),
        Style {
            bold: true,
            ..Style::default()
        },
    );
    let _ = styles.insert(
        "markup.underline".to_string(),
        Style {
            underline: true,
            ..Style::default()
        },
    );

    let _ = styles.insert("module.builtin".to_string(), fg(p.red));
    let _ = styles.insert("namespace.builtin".to_string(), fg(p.red));

    let _ = styles.insert("none".to_string(), Style::default());
    let _ = styles.insert("nospell".to_string(), fg_bg(p.fg, p.bg));

    let _ = styles.insert("number".to_string(), fg(p.orange));
    let _ = styles.insert("number.float".to_string(), fg(p.orange));

    let _ = styles.insert("operator".to_string(), fg(p.blue5));
    let _ = styles.insert("property".to_string(), fg(p.green1));

    let _ = styles.insert("punctuation".to_string(), fg(p.fg_dark));
    let _ = styles.insert("punctuation.bracket".to_string(), fg(p.fg_dark));
    let _ = styles.insert("punctuation.delimiter".to_string(), fg(p.blue5));
    let _ = styles.insert("punctuation.special".to_string(), fg(p.blue5));
    let _ = styles.insert("punctuation.special.markdown".to_string(), fg(p.orange));

    let _ = styles.insert("spell".to_string(), fg_bg(p.fg, p.bg));

    let _ = styles.insert("string".to_string(), fg(p.green));
    let _ = styles.insert("string.documentation".to_string(), fg(p.yellow));
    let _ = styles.insert("string.escape".to_string(), fg(p.magenta));
    let _ = styles.insert("string.regexp".to_string(), fg(p.blue6));
    let _ = styles.insert("string.special".to_string(), fg(p.blue1));
    insert_many_styles(
        &mut styles,
        &["string.special.key", "string.special.path"],
        fg(p.green1),
    );
    let _ = styles.insert("string.special.symbol".to_string(), fg(p.blue1));
    let _ = styles.insert(
        "string.special.url".to_string(),
        Style {
            fg: Some(p.teal),
            underline: true,
            ..Style::default()
        },
    );

    let _ = styles.insert("tag".to_string(), fg(p.red));
    let _ = styles.insert("tag.attribute".to_string(), fg(p.green1));
    let _ = styles.insert("tag.builtin".to_string(), fg(p.red));
    let _ = styles.insert("tag.delimiter".to_string(), fg(p.blue1));
    let _ = styles.insert("tag.delimiter.tsx".to_string(), tag_delimiter_tsx);
    let _ = styles.insert("tag.javascript".to_string(), fg(p.red));
    let _ = styles.insert("tag.tsx".to_string(), fg(p.red));

    let _ = styles.insert("type".to_string(), fg(p.blue1));
    let _ = styles.insert("type.builtin".to_string(), type_builtin);
    let _ = styles.insert("type.definition".to_string(), fg(p.blue1));

    let _ = styles.insert("variable".to_string(), fg(p.fg));
    let _ = styles.insert("variable.builtin".to_string(), fg(p.red));
    let _ = styles.insert("variable.member".to_string(), fg(p.green1));
    let _ = styles.insert("variable.parameter".to_string(), fg(p.yellow));
    let _ = styles.insert("variable.parameter.builtin".to_string(), parameter_builtin);

    let _ = ui.insert("default_fg".to_string(), fg(p.fg));
    let _ = ui.insert("default_bg".to_string(), bg(p.bg));
    let _ = ui.insert(
        "statusline".to_string(),
        fg_bg(p.fg_sidebar, p.bg_statusline),
    );
    let _ = ui.insert(
        "statusline_inactive".to_string(),
        fg_bg(p.fg_gutter, p.bg_statusline),
    );
    let _ = ui.insert("tab_active".to_string(), fg_bg(p.black, p.blue));
    let _ = ui.insert(
        "tab_inactive".to_string(),
        fg_bg(p.fg_gutter, p.bg_statusline),
    );
    let _ = ui.insert("selection".to_string(), bg(p.bg_visual));
    let _ = ui.insert("cursorline".to_string(), bg(p.bg_highlight));

    Theme::from_parts(styles, ui)
}
