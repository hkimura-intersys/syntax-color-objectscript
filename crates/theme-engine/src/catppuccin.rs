use crate::common::insert_many_styles;
use crate::theme_structures::{Rgb, Style, Theme};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub enum CatppuccinVariant {
    Latte,
    Frappe,
    Macchiato,
    Mocha,
}

#[derive(Debug, Clone, Copy)]
pub struct CatppuccinPalette {
    rosewater: Rgb,
    flamingo: Rgb,
    mauve: Rgb,
    red: Rgb,
    maroon: Rgb,
    peach: Rgb,
    yellow: Rgb,
    green: Rgb,
    teal: Rgb,
    sky: Rgb,
    blue: Rgb,
    lavender: Rgb,
    text: Rgb,
    overlay2: Rgb,
    overlay1: Rgb,
    overlay0: Rgb,
    surface1: Rgb,
    surface0: Rgb,
    base: Rgb,
    mantle: Rgb,
    crust: Rgb,
}

impl CatppuccinPalette {
    #[must_use]
    fn for_variant(variant: CatppuccinVariant) -> Self {
        match variant {
            CatppuccinVariant::Latte => Self::latte(),
            CatppuccinVariant::Frappe => Self::frappe(),
            CatppuccinVariant::Macchiato => Self::macchiato(),
            CatppuccinVariant::Mocha => Self::mocha(),
        }
    }

    #[must_use]
    pub const fn latte() -> Self {
        Self {
            rosewater: Rgb::new(0xdc, 0x8a, 0x78),
            flamingo: Rgb::new(0xdd, 0x78, 0x78),
            mauve: Rgb::new(0x88, 0x39, 0xef),
            red: Rgb::new(0xd2, 0x0f, 0x39),
            maroon: Rgb::new(0xe6, 0x45, 0x53),
            peach: Rgb::new(0xfe, 0x64, 0x0b),
            yellow: Rgb::new(0xdf, 0x8e, 0x1d),
            green: Rgb::new(0x40, 0xa0, 0x2b),
            teal: Rgb::new(0x17, 0x92, 0x99),
            sky: Rgb::new(0x04, 0xa5, 0xe5),
            blue: Rgb::new(0x1e, 0x66, 0xf5),
            lavender: Rgb::new(0x72, 0x87, 0xfd),
            text: Rgb::new(0x4c, 0x4f, 0x69),
            overlay2: Rgb::new(0x7c, 0x7f, 0x93),
            overlay1: Rgb::new(0x8c, 0x8f, 0xa1),
            overlay0: Rgb::new(0x9c, 0xa0, 0xb0),
            surface1: Rgb::new(0xbc, 0xc0, 0xcc),
            surface0: Rgb::new(0xcc, 0xd0, 0xda),
            base: Rgb::new(0xef, 0xf1, 0xf5),
            mantle: Rgb::new(0xe6, 0xe9, 0xef),
            crust: Rgb::new(0xdc, 0xe0, 0xe8),
        }
    }

    #[must_use]
    pub const fn frappe() -> Self {
        Self {
            rosewater: Rgb::new(0xf2, 0xd5, 0xcf),
            flamingo: Rgb::new(0xee, 0xbe, 0xbe),
            mauve: Rgb::new(0xca, 0x9e, 0xe6),
            red: Rgb::new(0xe7, 0x82, 0x84),
            maroon: Rgb::new(0xea, 0x99, 0x9c),
            peach: Rgb::new(0xef, 0x9f, 0x76),
            yellow: Rgb::new(0xe5, 0xc8, 0x90),
            green: Rgb::new(0xa6, 0xd1, 0x89),
            teal: Rgb::new(0x81, 0xc8, 0xbe),
            sky: Rgb::new(0x99, 0xd1, 0xdb),
            blue: Rgb::new(0x8c, 0xaa, 0xee),
            lavender: Rgb::new(0xba, 0xbb, 0xf1),
            text: Rgb::new(0xc6, 0xd0, 0xf5),
            overlay2: Rgb::new(0x94, 0x9c, 0xbb),
            overlay1: Rgb::new(0x83, 0x8b, 0xa7),
            overlay0: Rgb::new(0x73, 0x79, 0x94),
            surface1: Rgb::new(0x51, 0x57, 0x6d),
            surface0: Rgb::new(0x41, 0x45, 0x59),
            base: Rgb::new(0x30, 0x34, 0x46),
            mantle: Rgb::new(0x29, 0x2c, 0x3c),
            crust: Rgb::new(0x23, 0x26, 0x34),
        }
    }

    #[must_use]
    pub const fn macchiato() -> Self {
        Self {
            rosewater: Rgb::new(0xf4, 0xdb, 0xd6),
            flamingo: Rgb::new(0xf0, 0xc6, 0xc6),
            mauve: Rgb::new(0xc6, 0xa0, 0xf6),
            red: Rgb::new(0xed, 0x87, 0x96),
            maroon: Rgb::new(0xee, 0x99, 0xa0),
            peach: Rgb::new(0xf5, 0xa9, 0x7f),
            yellow: Rgb::new(0xee, 0xd4, 0x9f),
            green: Rgb::new(0xa6, 0xda, 0x95),
            teal: Rgb::new(0x8b, 0xd5, 0xca),
            sky: Rgb::new(0x91, 0xd7, 0xe3),
            blue: Rgb::new(0x8a, 0xad, 0xf4),
            lavender: Rgb::new(0xb7, 0xbd, 0xf8),
            text: Rgb::new(0xca, 0xd3, 0xf5),
            overlay2: Rgb::new(0x93, 0x9a, 0xb7),
            overlay1: Rgb::new(0x80, 0x87, 0xa2),
            overlay0: Rgb::new(0x6e, 0x73, 0x8d),
            surface1: Rgb::new(0x49, 0x4d, 0x64),
            surface0: Rgb::new(0x36, 0x3a, 0x4f),
            base: Rgb::new(0x24, 0x27, 0x3a),
            mantle: Rgb::new(0x1e, 0x20, 0x30),
            crust: Rgb::new(0x18, 0x19, 0x26),
        }
    }

    #[must_use]
    pub const fn mocha() -> Self {
        Self {
            rosewater: Rgb::new(0xf5, 0xe0, 0xdc),
            flamingo: Rgb::new(0xf2, 0xcd, 0xcd),
            mauve: Rgb::new(0xcb, 0xa6, 0xf7),
            red: Rgb::new(0xf3, 0x8b, 0xa8),
            maroon: Rgb::new(0xeb, 0xa0, 0xac),
            peach: Rgb::new(0xfa, 0xb3, 0x87),
            yellow: Rgb::new(0xf9, 0xe2, 0xaf),
            green: Rgb::new(0xa6, 0xe3, 0xa1),
            teal: Rgb::new(0x94, 0xe2, 0xd5),
            sky: Rgb::new(0x89, 0xdc, 0xeb),
            blue: Rgb::new(0x89, 0xb4, 0xfa),
            lavender: Rgb::new(0xb4, 0xbe, 0xfe),
            text: Rgb::new(0xcd, 0xd6, 0xf4),
            overlay2: Rgb::new(0x93, 0x99, 0xb2),
            overlay1: Rgb::new(0x7f, 0x84, 0x9c),
            overlay0: Rgb::new(0x6c, 0x70, 0x86),
            surface1: Rgb::new(0x45, 0x47, 0x5a),
            surface0: Rgb::new(0x31, 0x32, 0x44),
            base: Rgb::new(0x1e, 0x1e, 0x2e),
            mantle: Rgb::new(0x18, 0x18, 0x25),
            crust: Rgb::new(0x11, 0x11, 0x1b),
        }
    }
}

#[must_use]
pub fn build_catppuccin_theme(variant: CatppuccinVariant) -> Theme {
    let c = CatppuccinPalette::for_variant(variant);

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

    let normal_style = fg_bg(c.text, c.base);
    let keyword_style = fg(c.mauve);
    let conditional_style = Style {
        fg: Some(c.mauve),
        italic: true,
        ..Style::default()
    };
    let module_style = Style {
        fg: Some(c.yellow),
        italic: true,
        ..Style::default()
    };
    let comment_style = Style {
        fg: Some(c.overlay2),
        italic: true,
        ..Style::default()
    };

    let _ = styles.insert("normal".to_string(), normal_style);
    let _ = styles.insert(
        "selection".to_string(),
        Style {
            bg: Some(c.surface1),
            bold: true,
            ..Style::default()
        },
    );
    let _ = styles.insert("statusline".to_string(), fg_bg(c.text, c.mantle));
    let _ = styles.insert("ignore".to_string(), fg(c.overlay0));
    let _ = styles.insert("warning".to_string(), fg(c.yellow));
    let _ = styles.insert("error".to_string(), fg(c.red));

    insert_many_styles(
        &mut styles,
        &["annotation", "attribute", "attribute.builtin"],
        fg(c.peach),
    );
    let _ = styles.insert("boolean".to_string(), fg(c.peach));
    let _ = styles.insert("character".to_string(), fg(c.teal));
    insert_many_styles(
        &mut styles,
        &["character.printf", "character.special"],
        fg(c.mauve),
    );

    insert_many_styles(
        &mut styles,
        &["comment", "comment.documentation"],
        comment_style,
    );
    let _ = styles.insert("comment.error".to_string(), fg_bg(c.base, c.red));
    let _ = styles.insert("comment.warning".to_string(), fg_bg(c.base, c.yellow));
    let _ = styles.insert("comment.hint".to_string(), fg_bg(c.base, c.blue));
    let _ = styles.insert("comment.info".to_string(), fg_bg(c.base, c.blue));
    let _ = styles.insert("comment.todo".to_string(), fg_bg(c.base, c.flamingo));
    let _ = styles.insert("comment.note".to_string(), fg_bg(c.base, c.rosewater));

    let _ = styles.insert("conceal".to_string(), fg(c.overlay1));
    let _ = styles.insert("constant".to_string(), fg(c.peach));
    let _ = styles.insert("constant.builtin".to_string(), fg(c.peach));
    let _ = styles.insert("constant.macro".to_string(), fg(c.mauve));
    let _ = styles.insert("constructor".to_string(), fg(c.yellow));
    let _ = styles.insert("constructor.tsx".to_string(), fg(c.yellow));

    let _ = styles.insert("diff.plus".to_string(), fg(c.green));
    let _ = styles.insert("diff.minus".to_string(), fg(c.red));
    let _ = styles.insert("diff.delta".to_string(), fg(c.blue));

    let _ = styles.insert("embedded".to_string(), fg(c.text));

    insert_many_styles(
        &mut styles,
        &[
            "function",
            "function.call",
            "function.method",
            "function.method.call",
        ],
        fg(c.blue),
    );
    let _ = styles.insert("function.builtin".to_string(), fg(c.peach));
    let _ = styles.insert("function.macro".to_string(), fg(c.mauve));

    insert_many_styles(
        &mut styles,
        &[
            "keyword",
            "keyword.modifier",
            "keyword.type",
            "keyword.coroutine",
            "keyword.function",
            "keyword.import",
            "keyword.repeat",
            "keyword.return",
            "keyword.debug",
            "keyword.exception",
            "keyword.operator",
            "keyword.storage",
            "type.qualifier",
        ],
        keyword_style,
    );
    let _ = styles.insert("keyword.conditional".to_string(), conditional_style);
    let _ = styles.insert("keyword.conditional.ternary".to_string(), fg(c.sky));
    insert_many_styles(
        &mut styles,
        &["keyword.directive", "keyword.directive.define"],
        fg(c.mauve),
    );

    let _ = styles.insert("label".to_string(), fg(c.sky));

    let _ = styles.insert("markup".to_string(), fg(c.text));
    let _ = styles.insert("markup.heading".to_string(), fg(c.blue));
    let heading_rainbow = [c.red, c.peach, c.yellow, c.green, c.sky, c.lavender];
    for (index, color) in heading_rainbow.iter().enumerate() {
        let _ = styles.insert(format!("markup.heading.{}", index + 1), fg(*color));
    }
    let _ = styles.insert("markup.heading.7".to_string(), fg(c.blue));
    let _ = styles.insert("markup.heading.8".to_string(), fg(c.blue));
    let _ = styles.insert(
        "markup.italic".to_string(),
        Style {
            fg: Some(c.red),
            italic: true,
            ..Style::default()
        },
    );
    let _ = styles.insert("markup.link".to_string(), fg(c.lavender));
    insert_many_styles(
        &mut styles,
        &["markup.link.label", "markup.link.label.symbol"],
        fg(c.lavender),
    );
    let _ = styles.insert(
        "markup.link.url".to_string(),
        Style {
            fg: Some(c.blue),
            italic: true,
            underline: true,
            ..Style::default()
        },
    );
    let _ = styles.insert("markup.list".to_string(), fg(c.teal));
    let _ = styles.insert("markup.list.checked".to_string(), fg(c.green));
    let _ = styles.insert("markup.list.unchecked".to_string(), fg(c.overlay1));
    let _ = styles.insert("markup.list.markdown".to_string(), fg(c.teal));
    let _ = styles.insert("markup.math".to_string(), fg(c.blue));
    let _ = styles.insert("markup.quote".to_string(), fg(c.mauve));
    insert_many_styles(
        &mut styles,
        &["markup.raw", "markup.raw.block"],
        fg(c.green),
    );
    let _ = styles.insert("markup.raw.markdown_inline".to_string(), fg(c.flamingo));
    let _ = styles.insert("markup.strikethrough".to_string(), fg(c.text));
    let _ = styles.insert(
        "markup.strong".to_string(),
        Style {
            fg: Some(c.red),
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

    let _ = styles.insert("module".to_string(), module_style);
    let _ = styles.insert("module.builtin".to_string(), module_style);
    let _ = styles.insert("namespace.builtin".to_string(), module_style);

    let _ = styles.insert("none".to_string(), Style::default());
    let _ = styles.insert("nospell".to_string(), normal_style);
    let _ = styles.insert("spell".to_string(), normal_style);

    let _ = styles.insert("number".to_string(), fg(c.peach));
    let _ = styles.insert("number.float".to_string(), fg(c.peach));
    let _ = styles.insert("operator".to_string(), fg(c.sky));

    let _ = styles.insert("property".to_string(), fg(c.lavender));
    let _ = styles.insert("punctuation".to_string(), fg(c.overlay2));
    let _ = styles.insert("punctuation.bracket".to_string(), fg(c.overlay2));
    let _ = styles.insert("punctuation.delimiter".to_string(), fg(c.overlay2));
    let _ = styles.insert("punctuation.special".to_string(), fg(c.mauve));
    let _ = styles.insert("punctuation.special.markdown".to_string(), fg(c.mauve));

    let _ = styles.insert("string".to_string(), fg(c.green));
    let _ = styles.insert("string.documentation".to_string(), fg(c.teal));
    let _ = styles.insert("string.regexp".to_string(), fg(c.mauve));
    let _ = styles.insert("string.escape".to_string(), fg(c.mauve));
    let _ = styles.insert("string.special".to_string(), fg(c.mauve));
    let _ = styles.insert("string.special.key".to_string(), fg(c.mauve));
    let _ = styles.insert("string.special.path".to_string(), fg(c.mauve));
    let _ = styles.insert("string.special.symbol".to_string(), fg(c.flamingo));
    let _ = styles.insert(
        "string.special.url".to_string(),
        Style {
            fg: Some(c.blue),
            italic: true,
            underline: true,
            ..Style::default()
        },
    );

    let _ = styles.insert("tag".to_string(), fg(c.blue));
    let _ = styles.insert("tag.builtin".to_string(), fg(c.blue));
    let _ = styles.insert(
        "tag.attribute".to_string(),
        Style {
            fg: Some(c.yellow),
            italic: true,
            ..Style::default()
        },
    );
    let _ = styles.insert("tag.delimiter".to_string(), fg(c.teal));
    let _ = styles.insert("tag.delimiter.tsx".to_string(), fg(c.teal));
    let _ = styles.insert("tag.javascript".to_string(), fg(c.blue));
    let _ = styles.insert("tag.tsx".to_string(), fg(c.blue));

    let _ = styles.insert("type".to_string(), fg(c.yellow));
    let _ = styles.insert("type.builtin".to_string(), fg(c.mauve));
    let _ = styles.insert("type.definition".to_string(), fg(c.yellow));

    let _ = styles.insert("variable".to_string(), fg(c.text));
    let _ = styles.insert("variable.builtin".to_string(), fg(c.red));
    let _ = styles.insert("variable.member".to_string(), fg(c.lavender));
    let _ = styles.insert("variable.parameter".to_string(), fg(c.maroon));
    let _ = styles.insert("variable.parameter.builtin".to_string(), fg(c.maroon));

    let cursorline_bg = match variant {
        CatppuccinVariant::Latte => Rgb::blend(c.mantle, 0.70, c.base),
        _ => Rgb::blend(c.surface0, 0.64, c.base),
    };

    let _ = ui.insert("default_fg".to_string(), fg(c.text));
    let _ = ui.insert("default_bg".to_string(), bg(c.base));
    let _ = ui.insert("statusline".to_string(), fg_bg(c.text, c.mantle));
    let _ = ui.insert(
        "statusline_inactive".to_string(),
        fg_bg(c.surface1, c.mantle),
    );
    let _ = ui.insert("tab_active".to_string(), fg_bg(c.text, c.base));
    let _ = ui.insert("tab_inactive".to_string(), fg_bg(c.overlay0, c.crust));
    let _ = ui.insert(
        "selection".to_string(),
        Style {
            bg: Some(c.surface1),
            bold: true,
            ..Style::default()
        },
    );
    let _ = ui.insert("cursorline".to_string(), bg(cursorline_bg));

    Theme::from_parts(styles, ui)
}
