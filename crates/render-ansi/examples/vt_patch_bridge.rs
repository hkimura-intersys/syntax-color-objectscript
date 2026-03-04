use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};

use highlight_spans::{Grammar, SpanHighlighter};
use render_ansi::IncrementalRenderer;
use theme_engine::load_theme;

#[derive(Debug, Clone)]
struct Options {
    source_path: String,
    theme_name: String,
    grammar_name: String,
    width: usize,
    height: usize,
    previous_source_path: Option<String>,
}

fn parse_grammar(input: &str) -> Result<Grammar, String> {
    Grammar::from_name(input).ok_or_else(|| {
        format!(
            "unknown grammar '{}'; use one of: {}",
            input,
            Grammar::supported_names().join(", ")
        )
    })
}

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
            "--prev" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    return Err("expected value after --prev".to_string());
                };
                previous_source_path = Some(value.clone());
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
        previous_source_path,
    })
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  cargo run -p render-ansi --example vt_patch_bridge -- <source-file> [theme] [grammar] [--width N] [--height N] [--prev <old-source-file>]"
    );
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  cargo run -p render-ansi --example vt_patch_bridge -- sample.cls");
    eprintln!(
        "  cargo run -p render-ansi --example vt_patch_bridge -- sample.mac solarized-dark objectscript --width 200 --height 60"
    );
    eprintln!(
        "  cargo run -p render-ansi --example vt_patch_bridge -- new.mac tokyonight-dark objectscript --prev old.mac"
    );
}

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

    if let Some(previous_source_path) = &options.previous_source_path {
        let previous_source = fs::read(previous_source_path)?;
        let _ = renderer.highlight_to_patch(&mut highlighter, &previous_source, grammar, &theme)?;
    }

    let patch = renderer.highlight_to_patch(&mut highlighter, &source, grammar, &theme)?;
    print!("{patch}");
    io::stdout().flush()?;

    Ok(())
}
