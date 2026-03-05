# Renderer Modes Usage Guide

This guide shows how to use the three renderer modes in `render-ansi` and when to choose each one.

## Example UX-001: Full Rerender (Whole Frame)

**Audience:** Developers rendering a complete highlighted frame each update.
**Topic:** `highlight_to_ansi(...)` full-frame rendering.
**Goal:** Repaint the entire visible content in one write.

### Context

- When to use it:
  - Simpler integration where full repaint cost is acceptable.
  - Flows where complete frame output is easier than diff patching.
- When not to use it:
  - High-frequency edits where diff patching is needed.

### Prerequisites

- Dependencies:
  - `highlight-spans`, `theme-engine`, `render-ansi`
- Config:
  - Theme and grammar selection.
- Permissions:
  - Ability to write ANSI/VT bytes to the target terminal/PTY.

### Example (Minimal)

**Introduction:** This example highlights and renders a full frame every update.

```rust
use highlight_spans::Grammar;
use render_ansi::highlight_to_ansi;
use theme_engine::load_theme;

fn render_full(source: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let theme = load_theme("tokyonight-dark")?;
    let frame = highlight_to_ansi(source, Grammar::ObjectScript, &theme)?;
    Ok(frame)
}
```

**Explanation:**
1. Loads a theme once per render call.
2. Runs highlight + ANSI rendering across the full source.
3. Returns the whole ANSI string for repaint.

### Example (Advanced)

**Introduction:** This example keeps full-frame rendering but switches output color mode for constrained terminals.

```rust
use highlight_spans::Grammar;
use render_ansi::{highlight_to_ansi_with_mode, ColorMode};
use theme_engine::Theme;

fn render_full_ansi16(source: &[u8], theme: &Theme) -> Result<String, Box<dyn std::error::Error>> {
    let frame = highlight_to_ansi_with_mode(source, Grammar::ObjectScript, theme, ColorMode::Ansi16)?;
    Ok(frame)
}
```

**Explanation:**
1. Uses the same full-frame path.
2. Selects ANSI16 color mapping instead of truecolor.
3. Returns one complete frame string for repaint.

### Evidence

- `crates/render-ansi/src/lib.rs:492` (`highlight_to_ansi`)
- `crates/render-ansi/src/lib.rs:534` (`highlight_to_ansi_with_mode`)
- `crates/render-ansi/examples/show_highlight.rs:94` (real example usage)

### Validation Notes

- Verified signature against code: Yes
- Verified usage in tests or examples: Yes
- Mismatches or assumptions: None

## Example UX-002: Incremental Viewport Patch (Multiline)

**Audience:** Developers patching changed screen regions in multiline editors/terminals.
**Topic:** `IncrementalRenderer::highlight_to_patch(...)`.
**Goal:** Emit only changed VT sequences between snapshots.

### Context

- When to use it:
  - Multiline content where you know viewport width/height.
  - PTY/editor integrations that want minimal repaint bytes.
- When not to use it:
  - Unknown/unreliable terminal width.

### Prerequisites

- Dependencies:
  - `render-ansi` incremental API.
- Config:
  - `width`, `height`, and optional `origin`/`color_mode`.
- Permissions:
  - Stable output to the same terminal region each update.

### Example (Minimal)

**Introduction:** This example keeps renderer state and emits only diffs.

```rust
use highlight_spans::{Grammar, SpanHighlighter};
use render_ansi::IncrementalRenderer;
use theme_engine::load_theme;

fn incremental_loop(updates: impl Iterator<Item = String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut highlighter = SpanHighlighter::new()?;
    let theme = load_theme("tokyonight-dark")?;
    let mut renderer = IncrementalRenderer::new(120, 40);
    renderer.set_origin(1, 1);

    for snapshot in updates {
        let patch = renderer.highlight_to_patch(
            &mut highlighter,
            snapshot.as_bytes(),
            Grammar::ObjectScript,
            &theme,
        )?;
        if !patch.is_empty() {
            print!("{patch}");
        }
    }
    Ok(())
}
```

**Explanation:**
1. Keeps one renderer instance so previous-frame state persists.
2. Computes patch from old -> new snapshot.
3. Writes only changed sequences.

### Example (Advanced)

**Introduction:** This example seeds previous state and sets color mode.

```rust
use highlight_spans::Grammar;
use render_ansi::{ColorMode, IncrementalRenderer};

fn patch_once(
    renderer: &mut IncrementalRenderer,
    highlighter: &mut highlight_spans::SpanHighlighter,
    theme: &theme_engine::Theme,
    prev: &[u8],
    curr: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    renderer.set_color_mode(ColorMode::Ansi256);
    let _ = renderer.highlight_to_patch(highlighter, prev, Grammar::ObjectScript, theme)?;
    let patch = renderer.highlight_to_patch(highlighter, curr, Grammar::ObjectScript, theme)?;
    Ok(patch)
}
```

**Explanation:**
1. Seeds cache from prior snapshot.
2. Switches output mode when truecolor is unavailable.
3. Returns only the delta patch for `curr`.

### Evidence

- `crates/render-ansi/src/lib.rs:71` (`IncrementalRenderer`)
- `crates/render-ansi/src/lib.rs:166` (`highlight_to_patch`)
- `crates/render-ansi/examples/vt_patch_bridge.rs:199` (real example usage)

### Validation Notes

- Verified signature against code: Yes
- Verified usage in tests or examples: Yes
- Mismatches or assumptions: None

## Example UX-003: Stream Line Patch (Single Line, Width-Independent)

**Audience:** Developers patching a single command line/prompt input.
**Topic:** `StreamLineRenderer::highlight_line_to_patch(...)`.
**Goal:** Use relative cursor moves for single-line incremental repaint without width.

### Context

- When to use it:
  - Single editable line (shell-like input).
  - Terminal width is unknown/untrusted.
- When not to use it:
  - Multiline input (contains `\n`).

### Prerequisites

- Dependencies:
  - `render-ansi` stream-line API.
- Config:
  - Optional `ColorMode`.
- Permissions:
  - Cursor is on the same line and patch is written to same PTY line each update.

### Example (Minimal)

**Introduction:** This example patches one input line from `set x = 2` to `set x = 23`.

```rust
use highlight_spans::{Grammar, SpanHighlighter};
use render_ansi::StreamLineRenderer;
use theme_engine::load_theme;

fn stream_line_demo() -> Result<(), Box<dyn std::error::Error>> {
    let mut highlighter = SpanHighlighter::new()?;
    let theme = load_theme("tokyonight-dark")?;
    let mut renderer = StreamLineRenderer::new();

    let _ = renderer.highlight_line_to_patch(
        &mut highlighter,
        b"set x = 2",
        Grammar::ObjectScript,
        &theme,
    )?;

    let patch = renderer.highlight_line_to_patch(
        &mut highlighter,
        b"set x = 23",
        Grammar::ObjectScript,
        &theme,
    )?;

    print!("{patch}");
    Ok(())
}
```

**Explanation:**
1. Seeds previous single-line state.
2. Computes relative line patch to new text.
3. Emits only changed suffix/style transitions.

### Example (Advanced)

**Introduction:** This example uses fallback color mode and state reset.

```rust
use render_ansi::{ColorMode, StreamLineRenderer};

fn configure_stream(renderer: &mut StreamLineRenderer) {
    renderer.set_color_mode(ColorMode::Ansi16);
    renderer.clear_state();
}
```

**Explanation:**
1. Chooses 16-color output for constrained terminals.
2. Clears cache when host redraw invalidates prior line state.
3. Prevents diff drift after external line repaint.

### Evidence

- `crates/render-ansi/src/lib.rs:185` (`StreamLineRenderer`)
- `crates/render-ansi/src/lib.rs:239` (`highlight_line_to_patch`)
- `crates/render-ansi/src/lib.rs:225` (`MultiLineInput` behavior)
- `crates/render-ansi/examples/stream_line_bridge.rs:137` (real example usage)

### Validation Notes

- Verified signature against code: Yes
- Verified usage in tests or examples: Yes
- Mismatches or assumptions: None

## Doc/Code Mismatches

- None found while preparing these examples.
