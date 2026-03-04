use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};

use highlight_spans::{Grammar, SpanHighlighter};
use render_ansi::{ColorMode, IncrementalRenderer};
use theme_engine::load_theme;

#[derive(Debug, Clone)]
struct Options {
    source_path: String,
    theme_name: String,
    grammar_name: String,
    width: usize,
    height: usize,
    origin_row: usize,
    origin_col: usize,
    color_mode: ColorMode,
    previous_source_path: Option<String>,
}

/// Parses a grammar argument and returns a human-friendly error on failure.
fn parse_grammar(input: &str) -> Result<Grammar, String> {
    Grammar::from_name(input).ok_or_else(|| {
        format!(
            "unknown grammar '{}'; use one of: {}",
            input,
            Grammar::supported_names().join(", ")
        )
    })
}

/// Parses a viewport dimension from CLI value, env var, or fallback.
///
/// The resolved value is always at least `1`.
fn parse_dimension(value: Option<&str>, env_key: &str, fallback: usize) -> usize {
    if let Some(v) = value {
        if let Ok(parsed) = v.parse::<usize>() {
            return parsed.max(1);
        }
    }
    env::var(env_key)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .map(|v| v.max(1))
        .unwrap_or(fallback)
}

/// Parses a color mode argument and returns a human-friendly error on failure.
fn parse_color_mode(input: &str) -> Result<ColorMode, String> {
    ColorMode::from_name(input).ok_or_else(|| {
        format!(
            "unknown color mode '{}'; use one of: {}",
            input,
            ColorMode::supported_names().join(", ")
        )
    })
}

/// Parses CLI options for the `vt_patch_bridge` example.
///
/// Returns a user-facing error string suitable for usage output.
fn parse_args(args: &[String]) -> Result<Options, String> {
    if args.len() < 2 {
        return Err("missing source file".to_string());
    }

    let source_path = args[1].clone();
    let theme_name = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| "tokyonight-dark".to_string());
    let grammar_name = args
        .get(3)
        .cloned()
        .unwrap_or_else(|| "objectscript".to_string());

    let mut width_arg: Option<String> = None;
    let mut height_arg: Option<String> = None;
    let mut origin_row_arg: Option<String> = None;
    let mut origin_col_arg: Option<String> = None;
    let mut color_mode = ColorMode::TrueColor;
    let mut previous_source_path: Option<String> = None;
    let mut i = 4usize;
    while i < args.len() {
        match args[i].as_str() {
            "--width" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    return Err("expected value after --width".to_string());
                };
                width_arg = Some(value.clone());
            }
            "--height" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    return Err("expected value after --height".to_string());
                };
                height_arg = Some(value.clone());
            }
            "--origin-row" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    return Err("expected value after --origin-row".to_string());
                };
                origin_row_arg = Some(value.clone());
            }
            "--origin-col" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    return Err("expected value after --origin-col".to_string());
                };
                origin_col_arg = Some(value.clone());
            }
            "--prev" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    return Err("expected value after --prev".to_string());
                };
                previous_source_path = Some(value.clone());
            }
            "--color-mode" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    return Err("expected value after --color-mode".to_string());
                };
                color_mode = parse_color_mode(value)?;
            }
            flag => return Err(format!("unknown option '{flag}'")),
        }
        i += 1;
    }

    Ok(Options {
        source_path,
        theme_name,
        grammar_name,
        width: parse_dimension(width_arg.as_deref(), "COLUMNS", 240),
        height: parse_dimension(height_arg.as_deref(), "LINES", 80),
        origin_row: parse_dimension(origin_row_arg.as_deref(), "ORIGIN_ROW", 1),
        origin_col: parse_dimension(origin_col_arg.as_deref(), "ORIGIN_COL", 1),
        color_mode,
        previous_source_path,
    })
}

/// Prints CLI usage for the `vt_patch_bridge` example.
fn print_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  cargo run -p render-ansi --example vt_patch_bridge -- <source-file> [theme] [grammar] [--width N] [--height N] [--origin-row N] [--origin-col N] [--color-mode truecolor|ansi256|ansi16] [--prev <old-source-file>]"
    );
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  cargo run -p render-ansi --example vt_patch_bridge -- sample.cls");
    eprintln!(
        "  cargo run -p render-ansi --example vt_patch_bridge -- sample.mac solarized-dark objectscript --width 200 --height 60"
    );
    eprintln!(
        "  cargo run -p render-ansi --example vt_patch_bridge -- sample.mac tokyonight-dark objectscript --origin-row 4 --origin-col 7"
    );
    eprintln!(
        "  cargo run -p render-ansi --example vt_patch_bridge -- new.mac tokyonight-dark objectscript --prev old.mac"
    );
    eprintln!(
        "  cargo run -p render-ansi --example vt_patch_bridge -- sample.sql tokyonight-dark sql --color-mode ansi256"
    );
    eprintln!(
        "  cargo run -p render-ansi --example vt_patch_bridge -- sample.sql tokyonight-dark sql --color-mode ansi16"
    );
}

/// Emits an incremental VT patch for a highlighted source file.
///
/// When `--prev` is provided, the previous file seeds renderer state so output
/// contains only the delta from previous to current content.
///
/// # Errors
///
/// Returns an error when file IO, theme loading, or highlighting fails.
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let options = match parse_args(&args) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("invalid arguments: {err}");
            eprintln!();
            print_usage();
            return Ok(());
        }
    };

    let grammar =
        parse_grammar(&options.grammar_name).map_err(|msg| format!("invalid grammar: {msg}"))?;
    let source = fs::read(&options.source_path)?;
    let theme = load_theme(&options.theme_name)?;
    let mut highlighter = SpanHighlighter::new()?;
    let mut renderer = IncrementalRenderer::new(options.width, options.height);
    renderer.set_origin(options.origin_row, options.origin_col);
    renderer.set_color_mode(options.color_mode);

    if let Some(previous_source_path) = &options.previous_source_path {
        let previous_source = fs::read(previous_source_path)?;
        let _ = renderer.highlight_to_patch(&mut highlighter, &previous_source, grammar, &theme)?;
    }

    let patch = renderer.highlight_to_patch(&mut highlighter, &source, grammar, &theme)?;
    print!("{patch}");
    io::stdout().flush()?;

    Ok(())
}
