# render-ansi

`render-ansi` converts highlighted byte spans and resolved styles into ANSI/VT escaped terminal text.

## What It Provides

- End-to-end helpers:
  - `highlight_to_ansi(...) -> String`
  - `highlight_lines_to_ansi_lines(...) -> Vec<String>`
- Low-level render helpers:
  - `resolve_styled_spans(...)`
  - `render_ansi(...)`
  - `render_ansi_lines(...)`

## Quick Example

```rust
use highlight_spans::Grammar;
use render_ansi::highlight_to_ansi;
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = load_theme("tokyonight-dark")?;
    let out = highlight_to_ansi(b"set x = 42", Grammar::ObjectScript, &theme)?;
    println!("{out}");
    Ok(())
}
```

## When To Use This Crate

- Use this crate when you want terminal-ready ANSI output.
- If you have your own paint engine (for example native C/TUI), use `highlight-spans` + `theme-engine` directly and skip ANSI rendering.
