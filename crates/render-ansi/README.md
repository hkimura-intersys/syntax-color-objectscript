# render-ansi

`render-ansi` converts highlighted byte spans and resolved styles into ANSI/VT escaped terminal text.

## What It Provides

- End-to-end helpers:
  - `highlight_to_ansi(...) -> String`
  - `highlight_lines_to_ansi_lines(...) -> Vec<String>`
- Incremental patching:
  - `IncrementalRenderer::new(width, height)`
  - `IncrementalRenderer::render_patch(source, spans) -> String`
  - `IncrementalRenderer::highlight_to_patch(...) -> String`
  - `IncrementalSessionManager::new(default_width, default_height)` for multi-terminal/per-session state
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
    let out = highlight_to_ansi(b"SELECT 42", Grammar::Sql, &theme)?;
    println!("{out}");
    Ok(())
}
```

## When To Use This Crate

- Use this crate when you want terminal-ready ANSI output.
- If you have your own paint engine (for example native C/TUI), use `highlight-spans` + `theme-engine` directly and skip ANSI rendering.

## Examples

- `show_highlight`: full-frame ANSI render for a file.
- `zedit_bridge`: machine-readable paint ops (`start end fg_r fg_g fg_b bg_r bg_g bg_b flags`).
- `vt_patch_bridge`: incremental VT patch output using `IncrementalRenderer`.
