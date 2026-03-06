use std::env;
use std::error::Error;
use std::fs;

use highlight_spans::Grammar;
use render_ansi::{highlight_to_ansi_with_mode_and_background, ColorMode};
use theme_engine::load_theme;

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

/// Prints CLI usage for the `show_highlight` example.
fn print_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  cargo run -p render-ansi --example show_highlight -- <source-file> [theme] [grammar] [--color-mode truecolor|ansi256|ansi16] [--theme-bg]"
    );
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  cargo run -p render-ansi --example show_highlight -- sample.cls");
    eprintln!(
        "  cargo run -p render-ansi --example show_highlight -- sample.cls solarized-dark objectscript"
    );
    eprintln!(
        "  cargo run -p render-ansi --example show_highlight -- sample.sql tokyonight-dark sql"
    );
    eprintln!(
        "  cargo run -p render-ansi --example show_highlight -- sample.sql tokyonight-dark sql --color-mode ansi256"
    );
    eprintln!(
        "  cargo run -p render-ansi --example show_highlight -- sample.sql tokyonight-dark sql --color-mode ansi16"
    );
    eprintln!(
        "  cargo run -p render-ansi --example show_highlight -- sample.sql tokyonight-dark sql --theme-bg"
    );
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

/// Loads a file, highlights it, and prints full-frame ANSI output.
///
/// # Errors
///
/// Returns an error when argument parsing, file IO, theme loading, or
/// highlight/render execution fails.
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let source_path = &args[1];
    let theme_name = args.get(2).map(String::as_str).unwrap_or("tokyonight-dark");
    let grammar_name = args.get(3).map(String::as_str).unwrap_or("objectscript");
    let mut color_mode = ColorMode::TrueColor;
    let mut preserve_terminal_background = true;

    let mut i = 4usize;
    while i < args.len() {
        match args[i].as_str() {
            "--color-mode" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    return Err("expected value after --color-mode".into());
                };
                color_mode =
                    parse_color_mode(value).map_err(|msg| format!("invalid color mode: {msg}"))?;
            }
            "--theme-bg" => {
                preserve_terminal_background = false;
            }
            "--terminal-bg" => {
                preserve_terminal_background = true;
            }
            flag => {
                return Err(format!("unknown option '{flag}'").into());
            }
        }
        i += 1;
    }

    let grammar = parse_grammar(grammar_name).map_err(|msg| format!("invalid grammar: {msg}"))?;
    let source = fs::read(source_path)?;
    let theme = load_theme(theme_name)?;

    let rendered = highlight_to_ansi_with_mode_and_background(
        &source,
        grammar,
        &theme,
        color_mode,
        preserve_terminal_background,
    )?;
    println!("{rendered}");

    Ok(())
}
