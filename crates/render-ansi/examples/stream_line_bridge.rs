use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};

use highlight_spans::{Grammar, SpanHighlighter};
use render_ansi::{ColorMode, StreamLineRenderer};
use theme_engine::load_theme;

#[derive(Debug, Clone)]
struct Options {
    source_path: String,
    theme_name: String,
    grammar_name: String,
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

/// Parses CLI options for the `stream_line_bridge` example.
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

    let mut color_mode = ColorMode::TrueColor;
    let mut previous_source_path: Option<String> = None;
    let mut i = 4usize;
    while i < args.len() {
        match args[i].as_str() {
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
        color_mode,
        previous_source_path,
    })
}

/// Prints CLI usage for the `stream_line_bridge` example.
fn print_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  cargo run -p render-ansi --example stream_line_bridge -- <source-file> [theme] [grammar] [--color-mode truecolor|ansi256|ansi16] [--prev <old-source-file>]"
    );
    eprintln!();
    eprintln!("Notes:");
    eprintln!("  - Inputs must be single-line text files (no newline).");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  cargo run -p render-ansi --example stream_line_bridge -- new.sql");
    eprintln!(
        "  cargo run -p render-ansi --example stream_line_bridge -- new.sql tokyonight-dark sql --prev old.sql"
    );
    eprintln!(
        "  cargo run -p render-ansi --example stream_line_bridge -- new.sql tokyonight-dark sql --color-mode ansi16"
    );
}

/// Emits a stream-safe single-line VT patch for highlighted source.
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
    let mut renderer = StreamLineRenderer::new();
    renderer.set_color_mode(options.color_mode);

    if let Some(previous_source_path) = &options.previous_source_path {
        let previous_source = fs::read(previous_source_path)?;
        let _ = renderer.highlight_line_to_patch(
            &mut highlighter,
            &previous_source,
            grammar,
            &theme,
        )?;
    }

    let patch = renderer.highlight_line_to_patch(&mut highlighter, &source, grammar, &theme)?;
    print!("{patch}");
    io::stdout().flush()?;

    Ok(())
}
