use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};

use highlight_spans::{Grammar, SpanHighlighter};
use render_ansi::{
    highlight_to_ansi_with_highlighter_and_mode_and_background, ColorMode, IncrementalRenderer,
    RenderError, StreamLineRenderer,
};
use theme_engine::load_theme;

#[derive(Debug, Clone)]
struct Options {
    source_path: String,
    theme_name: String,
    grammar_name: String,
    width: usize,
    height: usize,
    origin_row: Option<usize>,
    origin_col: usize,
    color_mode: ColorMode,
    preserve_terminal_background: bool,
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

/// Maps stream-line renderer errors into user-facing CLI errors.
fn streamline_mode_error(err: RenderError) -> Box<dyn Error> {
    match err {
        RenderError::MultiLineInput => {
            "stream-line mode requires single-line input; pass --origin-row to use IncrementalRenderer"
                .into()
        }
        other => Box::new(other),
    }
}

/// Returns the number of logical lines in a snapshot (`\n` + 1), minimum `1`.
fn logical_line_count(source: &[u8]) -> usize {
    source.iter().filter(|&&byte| byte == b'\n').count() + 1
}

/// Builds a full-rerender patch that clears the previously painted logical block.
///
/// The patch uses relative movement only:
/// - return to column 1
/// - move up to the start of the old block
/// - clear each old line (`CSI 2K`)
/// - return to block start
/// - append the full rendered frame
fn build_full_rerender_patch(rendered: String, previous_source: Option<&[u8]>) -> String {
    let clear_lines = previous_source.map_or(1usize, logical_line_count).max(1);
    let mut patch = String::new();
    patch.push('\r');

    if clear_lines > 1 {
        patch.push_str(&format!("\x1b[{}A", clear_lines - 1));
    }

    for line_idx in 0..clear_lines {
        patch.push_str("\x1b[2K");
        if line_idx + 1 < clear_lines {
            patch.push_str("\x1b[1B\r");
        }
    }

    if clear_lines > 1 {
        patch.push_str(&format!("\x1b[{}A", clear_lines - 1));
    }
    patch.push('\r');
    patch.push_str(&rendered);
    patch
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
    let mut preserve_terminal_background = true;
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
            "--theme-bg" => {
                preserve_terminal_background = false;
            }
            "--terminal-bg" => {
                preserve_terminal_background = true;
            }
            flag => return Err(format!("unknown option '{flag}'")),
        }
        i += 1;
    }

    if origin_row_arg.is_none() && origin_col_arg.is_some() {
        return Err("--origin-col requires --origin-row".to_string());
    }

    let origin_row = match origin_row_arg.as_deref() {
        Some(value) => {
            let parsed = value
                .parse::<usize>()
                .map_err(|_| format!("invalid value '{value}' for --origin-row"))?;
            Some(parsed.max(1))
        }
        None => None,
    };
    let origin_col = match origin_col_arg.as_deref() {
        Some(value) => value
            .parse::<usize>()
            .map(|v| v.max(1))
            .map_err(|_| format!("invalid value '{value}' for --origin-col"))?,
        None => 1,
    };

    Ok(Options {
        source_path,
        theme_name,
        grammar_name,
        width: parse_dimension(width_arg.as_deref(), "COLUMNS", 240),
        height: parse_dimension(height_arg.as_deref(), "LINES", 80),
        origin_row,
        origin_col,
        color_mode,
        preserve_terminal_background,
        previous_source_path,
    })
}

/// Prints CLI usage for the `vt_patch_bridge` example.
fn print_usage() {
    eprintln!("Usage:");
    eprintln!(
        "  cargo run -p render-ansi --example vt_patch_bridge -- <source-file> [theme] [grammar] [--color-mode truecolor|ansi256|ansi16] [--theme-bg] [--prev <old-source-file>] [--origin-row N --origin-col N --width N --height N]"
    );
    eprintln!();
    eprintln!("Mode selection:");
    eprintln!(
        "  - no --origin-row: uses StreamLineRenderer (single-line relative mode), falls back to full render for multiline input"
    );
    eprintln!("  - with --origin-row: uses IncrementalRenderer (multiline viewport mode)");
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
    eprintln!(
        "  cargo run -p render-ansi --example vt_patch_bridge -- sample.sql tokyonight-dark sql --theme-bg"
    );
}

/// Emits a VT patch for a highlighted source file.
///
/// When `--prev` is provided, the previous file seeds renderer state so output
/// contains only the delta from previous to current content.
///
/// Mode selection:
/// - no `--origin-row`: `StreamLineRenderer` (falls back to full render for multiline snapshots)
/// - with `--origin-row`: `IncrementalRenderer`
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
    let previous_source = if let Some(previous_source_path) = &options.previous_source_path {
        Some(fs::read(previous_source_path)?)
    } else {
        None
    };
    let theme = load_theme(&options.theme_name)?;
    let mut highlighter = SpanHighlighter::new()?;

    let patch = if let Some(origin_row) = options.origin_row {
        let mut renderer = IncrementalRenderer::new(options.width, options.height);
        renderer.set_origin(origin_row, options.origin_col);
        renderer.set_color_mode(options.color_mode);
        renderer.set_preserve_terminal_background(options.preserve_terminal_background);

        if let Some(previous_source) = previous_source.as_deref() {
            let _ =
                renderer.highlight_to_patch(&mut highlighter, previous_source, grammar, &theme)?;
        }

        renderer.highlight_to_patch(&mut highlighter, &source, grammar, &theme)?
    } else {
        let source_is_multiline = source.contains(&b'\n');
        let previous_is_multiline = previous_source
            .as_ref()
            .is_some_and(|snapshot| snapshot.contains(&b'\n'));

        if source_is_multiline || previous_is_multiline {
            let rendered = highlight_to_ansi_with_highlighter_and_mode_and_background(
                &mut highlighter,
                &source,
                grammar,
                &theme,
                options.color_mode,
                options.preserve_terminal_background,
            )?;
            build_full_rerender_patch(rendered, previous_source.as_deref())
        } else {
            let mut renderer = StreamLineRenderer::new();
            renderer.set_color_mode(options.color_mode);
            renderer.set_preserve_terminal_background(options.preserve_terminal_background);

            if let Some(previous_source) = previous_source.as_deref() {
                let _ = renderer
                    .highlight_line_to_patch(&mut highlighter, previous_source, grammar, &theme)
                    .map_err(streamline_mode_error)?;
            }

            renderer
                .highlight_line_to_patch(&mut highlighter, &source, grammar, &theme)
                .map_err(streamline_mode_error)?
        }
    };

    print!("{patch}");
    io::stdout().flush()?;

    Ok(())
}
