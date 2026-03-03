use std::env;
use std::error::Error;
use std::fs;

use highlight_spans::{Grammar, SpanHighlighter};
use theme_engine::{load_theme, Rgb, Style, Theme};

const FLAG_BOLD: u8 = 0b0001;
const FLAG_ITALIC: u8 = 0b0010;
const FLAG_UNDERLINE: u8 = 0b0100;
const FLAG_HAS_BG: u8 = 0b1000;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CPaintOp {
    pub start_byte: u32,
    pub end_byte: u32,
    pub fg_r: u8,
    pub fg_g: u8,
    pub fg_b: u8,
    pub bg_r: u8,
    pub bg_g: u8,
    pub bg_b: u8,
    pub flags: u8,
}

fn parse_grammar(input: &str) -> Result<Grammar, String> {
    match input.trim().to_ascii_lowercase().as_str() {
        "objectscript"
        | "os"
        | "playground"
        | "objectscriptplayground"
        | "objectscript_playground" => Ok(Grammar::ObjectScript),
        _ => Err(format!(
            "unknown grammar '{}'; use objectscript (aliases: os, objectscript_playground, playground)",
            input
        )),
    }
}

fn merge_with_normal(style: Option<Style>, normal: Style) -> Style {
    let style = style.unwrap_or_default();
    Style {
        fg: style.fg.or(normal.fg),
        bg: style.bg.or(normal.bg),
        bold: style.bold || normal.bold,
        italic: style.italic || normal.italic,
        underline: style.underline || normal.underline,
    }
}

fn style_to_c_op(
    span_start: usize,
    span_end: usize,
    style: Style,
) -> Result<CPaintOp, Box<dyn Error>> {
    let start_byte = u32::try_from(span_start)?;
    let end_byte = u32::try_from(span_end)?;

    let fg = style.fg.unwrap_or(Rgb::new(220, 220, 220));
    let bg = style.bg.unwrap_or(Rgb::new(0, 0, 0));

    let mut flags = 0u8;
    if style.bold {
        flags |= FLAG_BOLD;
    }
    if style.italic {
        flags |= FLAG_ITALIC;
    }
    if style.underline {
        flags |= FLAG_UNDERLINE;
    }
    if style.bg.is_some() {
        flags |= FLAG_HAS_BG;
    }

    Ok(CPaintOp {
        start_byte,
        end_byte,
        fg_r: fg.r,
        fg_g: fg.g,
        fg_b: fg.b,
        bg_r: bg.r,
        bg_g: bg.g,
        bg_b: bg.b,
        flags,
    })
}

fn build_c_paint_ops(
    source: &[u8],
    grammar: Grammar,
    theme: &Theme,
    highlighter: &mut SpanHighlighter,
) -> Result<Vec<CPaintOp>, Box<dyn Error>> {
    let result = highlighter.highlight(source, grammar)?;

    let normal = theme.resolve("normal").copied().unwrap_or(Style {
        fg: Some(Rgb::new(220, 220, 220)),
        ..Style::default()
    });

    let mut ops = Vec::with_capacity(result.spans.len());
    for span in &result.spans {
        let capture = result
            .attrs
            .get(span.attr_id)
            .map(|a| a.capture_name.as_str())
            .unwrap_or("normal");

        let resolved = merge_with_normal(theme.resolve(capture).copied(), normal);
        let op = style_to_c_op(span.start_byte, span.end_byte, resolved)?;
        ops.push(op);
    }

    Ok(ops)
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  cargo run -p render-ansi --example zedit_bridge -- <source-file> [theme] [grammar]"
    );
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  cargo run -p render-ansi --example zedit_bridge -- sample.cls");
    eprintln!(
        "  cargo run -p render-ansi --example zedit_bridge -- sample.mac solarized-dark objectscript"
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let source_path = &args[1];
    let theme_name = args.get(2).map(String::as_str).unwrap_or("tokyonight-dark");
    let grammar_name = args.get(3).map(String::as_str).unwrap_or("objectscript");

    let grammar = parse_grammar(grammar_name).map_err(|msg| format!("invalid grammar: {msg}"))?;
    let source = fs::read(source_path)?;
    let theme = load_theme(theme_name)?;
    let mut highlighter = SpanHighlighter::new()?;
    let ops = build_c_paint_ops(&source, grammar, &theme, &mut highlighter)?;

    // Print a stable, machine-readable format:
    // start end fg_r fg_g fg_b bg_r bg_g bg_b flags
    for op in ops {
        println!(
            "{} {} {} {} {} {} {} {} {}",
            op.start_byte,
            op.end_byte,
            op.fg_r,
            op.fg_g,
            op.fg_b,
            op.bg_r,
            op.bg_g,
            op.bg_b,
            op.flags
        );
    }

    Ok(())
}
