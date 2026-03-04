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

## Exact Usage: Multiple IRIS Sessions

Use one renderer state per terminal session:

```rust
use highlight_spans::{Grammar, SpanHighlighter};
use render_ansi::IncrementalSessionManager;
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut highlighter = SpanHighlighter::new()?;
    let theme = load_theme("tokyonight-dark")?;
    let mut sessions = IncrementalSessionManager::new(120, 40);

    sessions.ensure_session("iris-a").set_origin(4, 7);
    sessions.ensure_session("iris-b").set_origin(4, 7);

    let events = vec![
        ("iris-a", "set x = 2\n"),
        ("iris-b", "set y = 10\n"),
        ("iris-a", "set x = 23\n"),
    ];

    for (session_id, snapshot) in events {
        let patch = sessions.highlight_to_patch_for_session(
            session_id,
            &mut highlighter,
            snapshot.as_bytes(),
            Grammar::ObjectScript,
            &theme,
        )?;
        if !patch.is_empty() {
            // Write patch to the matching terminal session PTY.
        }
    }

    Ok(())
}
```

## Exact Usage: `vt_patch_bridge` CLI

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

## Using This in a Live IRIS Terminal Loop

1. Track one renderer per IRIS PTY/session.
2. Set initial `width`, `height`, and `origin`.
3. For each user edit or prompt refresh, capture the current editable snapshot.
4. Call `highlight_to_patch(...)` / `highlight_to_patch_for_session(...)`.
5. Write returned patch bytes to the same PTY.
6. On resize: call `renderer.resize(new_width, new_height)`.
7. On full screen clear/repaint by host: call `renderer.clear_state()` and re-prime.

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
- If a session is redrawn externally without telling the renderer, call `clear_state()` to resynchronize.
