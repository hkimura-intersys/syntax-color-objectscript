# render-ansi

`render-ansi` converts highlighted byte spans and resolved styles into ANSI/VT escaped terminal text.

## What It Provides

- End-to-end helpers:
  - `highlight_to_ansi(...) -> String`
  - `highlight_to_ansi_with_mode(..., ColorMode::{TrueColor|Ansi256|Ansi16}) -> String`
  - `highlight_to_ansi_with_mode_and_background(..., color_mode, preserve_terminal_background) -> String`
  - `highlight_lines_to_ansi_lines(...) -> Vec<String>`
  - `highlight_lines_to_ansi_lines_with_mode(..., ColorMode::{TrueColor|Ansi256|Ansi16}) -> Vec<String>`
  - `highlight_lines_to_ansi_lines_with_mode_and_background(..., color_mode, preserve_terminal_background) -> Vec<String>`
  - High-level highlight helpers treat `normal` as a base layer and fill uncovered ranges with that style.
  - Background behavior is configurable; default preserves terminal background (`preserve_terminal_background = true`).
- Incremental patching:
  - `IncrementalRenderer::new(width, height)`
  - `IncrementalRenderer::set_origin(row, col)`
  - `IncrementalRenderer::set_color_mode(ColorMode::{TrueColor|Ansi256|Ansi16})`
  - `IncrementalRenderer::set_preserve_terminal_background(bool)`
  - `IncrementalRenderer::render_patch(source, spans) -> String`
  - `IncrementalRenderer::highlight_to_patch(...) -> String`
- Stream-safe single-line diff (no width/XY assumptions):
  - `StreamLineRenderer::new()`
  - `StreamLineRenderer::set_preserve_terminal_background(bool)`
  - `StreamLineRenderer::render_line_patch(source, spans) -> String`
  - `StreamLineRenderer::highlight_line_to_patch(...) -> String`
- Low-level render helpers:
  - `resolve_styled_spans(...)`
  - `render_ansi(...)`
  - `render_ansi_lines(...)`
- Terminal default-color helpers (OSC):
  - `osc_set_default_colors_from_theme(&theme) -> String` (OSC 10/11)
  - `osc_reset_default_colors() -> &'static str` (OSC 110/111)

## Quick Example

```rust
use highlight_spans::Grammar;
use render_ansi::{highlight_to_ansi_with_mode_and_background, ColorMode};
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = load_theme("tokyonight-dark")?;
    let out = highlight_to_ansi_with_mode_and_background(
        b"SELECT 42",
        Grammar::Sql,
        &theme,
        ColorMode::Ansi256,
        false, // use theme background colors
    )?;
    println!("{out}");
    Ok(())
}
```

## When To Use This Crate

- Use this crate when you want terminal-ready ANSI output.
- Incremental patching computes cursor columns by display width (grapheme-aware), not raw bytes.
- `StreamLineRenderer` uses relative single-line patching (`CUB` + overwrite + optional `EL`) and is useful when terminal width is unknown.
- Tab cells are interpreted with an 8-column tab stop in the incremental path.
- Default ANSI color mode is truecolor (`38;2;r;g;b`). Use `Ansi256` or `Ansi16` for constrained terminals.
- Default background mode preserves terminal background; disable it to emit themed background colors.
- For terminals that support OSC 10/11, you can set session default fg/bg from theme `default_fg`/`default_bg` (fallback `normal`) at session start and reset at session end.
- In `vt_patch_bridge` multiline fallback mode (no `--origin-row`), the bridge clears/repositions using logical `\n` line count from `--prev` when provided.
- If you have your own paint engine (for example native C), use `highlight-spans` + `theme-engine` directly and skip ANSI rendering.

## Examples

- `show_highlight`: full-frame ANSI render for a file.
- `zedit_bridge`: machine-readable paint ops (`start end fg_r fg_g fg_b bg_r bg_g bg_b flags`).
- `vt_patch_bridge`: automatic bridge mode selection:
  - with `--origin-row`: `IncrementalRenderer` (multiline viewport diff)
  - without `--origin-row` + single-line snapshots: `StreamLineRenderer`
  - without `--origin-row` + multiline snapshots: full-rerender fallback with relative clear/reposition
- `stream_line_bridge`: width-independent single-line patch output using `StreamLineRenderer`.

## Incremental Origin Offset

If your editable text starts after a prompt (or inside a viewport sub-region), set an origin:

```rust
let mut renderer = render_ansi::IncrementalRenderer::new(120, 40);
renderer.set_origin(4, 7); // row 4, col 7 (1-based)
```

For the CLI bridge:

```bash
cargo run -p render-ansi --example vt_patch_bridge -- \
  /path/to/file.sql tokyonight-dark sql \
  --origin-row 4 --origin-col 7
```

Automatic bridge mode examples:

```bash
# Single-line snapshots (no origin): stream-line relative patch mode
cargo run -p render-ansi --example vt_patch_bridge -- \
  new.mac tokyonight-dark objectscript \
  --prev old.mac

# Multiline snapshots (no origin): full-rerender fallback with clear/reposition
cargo run -p render-ansi --example vt_patch_bridge -- \
  /tmp/iris-new.sql tokyonight-dark sql \
  --prev /tmp/iris-old.sql
```
