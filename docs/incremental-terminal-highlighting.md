# Incremental Terminal Syntax Highlighting Guide

This guide explains the incremental syntax-highlighting path end-to-end and shows exactly how to use it.

## What Incremental Means Here

Instead of repainting the full frame on every edit, `render-ansi`:

1. Highlights the latest text snapshot.
2. Converts highlights to styled cells.
3. Diffs against the previous styled snapshot.
4. Emits only the VT patch needed to update changed cells.

That patch includes cursor movement (`CUP`), style changes (`SGR`), plain text, and optional erase-to-end-of-line (`EL`).

## Pipeline (Code-Level)

For each update cycle:

1. `SpanHighlighter::highlight(...)`
2. `resolve_styled_spans(...)`
3. `IncrementalRenderer::render_patch(...)`
4. `diff_lines_to_patch(...)`
5. host writes patch bytes to terminal PTY

Important behavior:

- Patch coordinates are offset by `set_origin(row, col)` if configured.
- Incremental column math is display-width based (grapheme-aware), not byte-count based.
- Tabs use an 8-column tab stop in incremental mode.
- ANSI color mode defaults to truecolor (`38;2;r;g;b`); set `Ansi256` or `Ansi16` for constrained terminals.

## Prerequisites

- `highlight-spans`, `theme-engine`, `render-ansi`
- A terminal/PTY you control (same output surface each cycle)
- A stable way to capture the editable buffer snapshot per update

## Exact Usage: Single Session (Rust API)

```rust
use std::io::{self, Write};

use highlight_spans::{Grammar, SpanHighlighter};
use render_ansi::{ColorMode, IncrementalRenderer};
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut highlighter = SpanHighlighter::new()?;
    let theme = load_theme("tokyonight-dark")?;

    // Viewport used for clipping.
    let mut renderer = IncrementalRenderer::new(120, 40);
    renderer.set_color_mode(ColorMode::Ansi16); // optional fallback for 16-color terminals

    // Editable region starts at row 4, col 7 (for example after a prompt).
    renderer.set_origin(4, 7);

    // Prime previous state (old frame).
    let old_snapshot = "SELECT Name, DOB\nFROM Sample.Person\nWHERE Home_State = :state\nORDER BY Name\n";
    let _ = renderer.highlight_to_patch(
        &mut highlighter,
        old_snapshot.as_bytes(),
        Grammar::Sql,
        &theme,
    )?;

    // New frame.
    let new_snapshot = "SELECT Name\nFROM Sample.Person\nWHERE Home_State = :state\nORDER BY Name\n";
    let patch = renderer.highlight_to_patch(
        &mut highlighter,
        new_snapshot.as_bytes(),
        Grammar::Sql,
        &theme,
    )?;

    if !patch.is_empty() {
        print!("{patch}");
        io::stdout().flush()?;
    }

    Ok(())
}
```

Notes:

- First call usually emits a full paint for that region.
- Next calls emit only deltas.
- If no visual change: patch is empty.

## Exact Usage: IRIS READ Lifecycle (Direct + SQL Shell)

In IRIS, Direct mode and SQL shell input are sequential `READ` loops, not concurrent
editable sessions. A `READ` completes on ENTER, then the next `READ` starts afterward
(often a few milliseconds later). Keep one renderer for the active `READ`, and pick
grammar from your `$ZU()` mode signal (`DIRECT` vs SQL).

```rust
use highlight_spans::{Grammar, SpanHighlighter};
use render_ansi::IncrementalRenderer;
use theme_engine::load_theme;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IrisReadMode {
    Direct,
    SqlShell,
}

fn grammar_for_mode(mode: IrisReadMode) -> Grammar {
    match mode {
        IrisReadMode::Direct => Grammar::ObjectScript,
        IrisReadMode::SqlShell => Grammar::Sql,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut highlighter = SpanHighlighter::new()?;
    let theme = load_theme("tokyonight-dark")?;
    let mut renderer = IncrementalRenderer::new(120, 40);
    let mut active_read_id: Option<u64> = None;

    // `mode` comes from `$ZU()` at READ start.
    let events = vec![
        (10_u64, IrisReadMode::Direct, "set x = 2\n"),
        (10_u64, IrisReadMode::Direct, "set x = 23\n"),
        // Previous READ completed on ENTER; SQL shell READ starts next.
        (11_u64, IrisReadMode::SqlShell, "SELECT Name, DOB\n"),
        (11_u64, IrisReadMode::SqlShell, "SELECT Name\n"),
    ];

    for (read_id, mode, snapshot) in events {
        if active_read_id != Some(read_id) {
            active_read_id = Some(read_id);
            renderer.clear_state(); // do not diff across different READ lifecycles
            renderer.set_origin(4, 7); // set prompt-relative origin for this READ
        }

        let patch = renderer.highlight_to_patch(
            &mut highlighter,
            snapshot.as_bytes(),
            grammar_for_mode(mode),
            &theme,
        )?;
        if !patch.is_empty() {
            // Write patch to the same IRIS PTY.
        }
    }

    Ok(())
}
```

## Exact Usage: Multiple Terminal PTYs (Manual State Map)

If your host multiplexes independent PTYs/connections, keep one
`IncrementalRenderer` per terminal ID in your own map.

```rust
use std::collections::HashMap;

use highlight_spans::{Grammar, SpanHighlighter};
use render_ansi::IncrementalRenderer;
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut highlighter = SpanHighlighter::new()?;
    let theme = load_theme("tokyonight-dark")?;
    let mut renderers: HashMap<String, IncrementalRenderer> = HashMap::new();

    let events = vec![
        ("pty-a", "set x = 2\n"),
        ("pty-b", "set y = 10\n"),
        ("pty-a", "set x = 23\n"),
    ];

    for (pty_id, snapshot) in events {
        let renderer = renderers.entry(pty_id.to_string()).or_insert_with(|| {
            let mut r = IncrementalRenderer::new(120, 40);
            r.set_origin(4, 7);
            r
        });

        let patch = renderer.highlight_to_patch(
            &mut highlighter,
            snapshot.as_bytes(),
            Grammar::ObjectScript,
            &theme,
        )?;
        if !patch.is_empty() {
            // Write patch to the matching PTY.
        }
    }

    Ok(())
}
```

## Exact Usage: `vt_patch_bridge` CLI

`vt_patch_bridge` now auto-selects renderer mode:

1. With `--origin-row`: uses `IncrementalRenderer` (viewport-aware multiline diff).
2. Without `--origin-row` and single-line snapshots: uses `StreamLineRenderer`.
3. Without `--origin-row` and multiline snapshots: falls back to full render and emits relative clear/reposition before repaint.

Prepare old/new snapshots:

```bash
cat > /tmp/iris-old.sql <<'EOF'
SELECT Name, DOB
FROM Sample.Person
WHERE Home_State = :state
ORDER BY Name
EOF

cat > /tmp/iris-new.sql <<'EOF'
SELECT Name
FROM Sample.Person
WHERE Home_State = :state
ORDER BY Name
EOF
```

Generate patch:

```bash
cargo run -p render-ansi --example vt_patch_bridge -- \
  /tmp/iris-new.sql tokyonight-dark sql \
  --prev /tmp/iris-old.sql \
  --width 120 --height 40 \
  --origin-row 4 --origin-col 7 \
  --color-mode ansi16
```

If you want to inspect raw escapes safely instead of applying them to your current shell screen:

```bash
cargo run -q -p render-ansi --example vt_patch_bridge -- \
  /tmp/iris-new.sql tokyonight-dark sql \
  --prev /tmp/iris-old.sql \
  --width 120 --height 40 \
  --origin-row 4 --origin-col 7 \
  --color-mode ansi16 | sed -n l
```

Single-line auto mode (no origin):

```bash
cat > old.mac <<'EOF'
set x = 2
EOF

cat > new.mac <<'EOF'
set x = 23
EOF

cargo run -p render-ansi --example vt_patch_bridge -- \
  new.mac tokyonight-dark objectscript \
  --prev old.mac
```

Multiline auto-fallback mode (no origin):

```bash
cargo run -p render-ansi --example vt_patch_bridge -- \
  /tmp/iris-new.sql tokyonight-dark sql \
  --prev /tmp/iris-old.sql
```

## Using This in a Live IRIS Terminal Loop

1. Track one `IncrementalRenderer` for the active IRIS `READ`.
2. At each `READ` start, inspect your `$ZU()` mode signal and map it to grammar (`DIRECT` -> `ObjectScript`, SQL shell -> `Sql`).
3. On `READ` boundary (ENTER completed one loop, next loop starts), call `renderer.clear_state()`, then set `origin`.
4. For each prompt refresh/edit within that `READ`, capture the current snapshot and call `highlight_to_patch(...)`.
5. Write returned patch bytes to the same PTY.
6. On resize: call `renderer.resize(new_width, new_height)`.
7. If you truly multiplex multiple PTYs, keep a `HashMap<String, IncrementalRenderer>` keyed by PTY/session ID.

## Fallback: Width-Independent Single-Line Patches

If terminal width is unknown or unreliable, use `StreamLineRenderer` for a single
editable line. It diffs with relative cursor-left (`CUB`) + overwrite + optional
erase-to-end-of-line (`EL`) and avoids absolute XY movement.

```rust
use highlight_spans::{Grammar, SpanHighlighter};
use render_ansi::StreamLineRenderer;
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut highlighter = SpanHighlighter::new()?;
    let theme = load_theme("tokyonight-dark")?;
    let mut renderer = StreamLineRenderer::new();

    let _ = renderer.highlight_line_to_patch(
        &mut highlighter,
        b"SELECT Name FROM Sample.Person",
        Grammar::Sql,
        &theme,
    )?;
    let patch = renderer.highlight_line_to_patch(
        &mut highlighter,
        b"SELECT ID FROM Sample.Person",
        Grammar::Sql,
        &theme,
    )?;

    if !patch.is_empty() {
        // Write patch to the same line/terminal stream.
    }
    Ok(())
}
```

Notes:

- `StreamLineRenderer` expects single-line input (no `\n`).
- Cursor must remain at end-of-line between updates.
- This mode is safer for unknown wrap behavior but only supports line-local updates.
- In `vt_patch_bridge`, multiline snapshots without `--origin-row` do not error; they use a full-rerender fallback patch.

## How To Determine Origin (`row`, `col`)

Common approaches:

- You control prompt rendering: compute origin directly.
- Query terminal cursor position (`ESC[6n`) right before user input starts.
- Use your host UI layout model if terminal content is virtualized.

Origin must be terminal-global coordinates for where your editable buffer begins.

## Common Pitfalls

- Running incremental patch output directly in your normal shell prompt can overwrite visible prompt/history text. Use a controlled region/PTY.
- Byte offsets from parsers are not terminal columns. Incremental renderer already converts to display-width columns.
- If host wrap behavior differs from your provided `width`, patches can drift.
- In auto-fallback full rerender mode (no `--origin-row` + multiline), clearing is based on logical `\n` lines, not terminal-wrapped rows.
- If you diff across different IRIS `READ` lifecycles (for example Direct -> SQL shell) without `clear_state()`, patch output can drift.
- If a screen region is redrawn externally without telling the renderer, call `clear_state()` to resynchronize.
- If you cannot trust terminal width/wrap, prefer `StreamLineRenderer` (line-local mode) or full-frame rerender with `highlight_to_ansi`.
