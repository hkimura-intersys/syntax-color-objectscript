use std::env;
use std::error::Error;
use std::fs;

use highlight_spans::Grammar;
use render_ansi::highlight_to_ansi;
use theme_engine::load_theme;

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

fn print_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  cargo run -p render-ansi --example show_highlight -- <source-file> [theme] [grammar]"
    );
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  cargo run -p render-ansi --example show_highlight -- sample.cls");
    eprintln!(
        "  cargo run -p render-ansi --example show_highlight -- sample.cls solarized-dark objectscript"
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

    let rendered = highlight_to_ansi(&source, grammar, &theme)?;
    println!("{rendered}");

    Ok(())
}
