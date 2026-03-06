# Integration Steps
## 1) With renderer (`render-ansi`)

### Step 1: Add dependencies

```toml
[dependencies]
highlight-spans = { path = "../syntax-color-objectscript/crates/highlight-spans" }
theme-engine = { path = "../syntax-color-objectscript/crates/theme-engine" }
render-ansi = { path = "../syntax-color-objectscript/crates/render-ansi" }
```

### Step 2: Use the one-shot rendering API

```rust
use highlight_spans::Grammar;
use render_ansi::highlight_to_ansi;
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = b"set x = 42";
    let theme = load_theme("tokyonight-dark")?;
    let ansi = highlight_to_ansi(source, Grammar::ObjectScript, &theme)?;
    println!("{ansi}");
    Ok(())
}
```

### Step 3: Run

```bash
cargo run
```

### Optional: line-oriented output for incremental redraw

Use `highlight_lines_to_ansi_lines(...)` when your terminal host redraws line-by-line.

### Optional: incremental VT patch output for interactive IRIS sessions

Use `IncrementalRenderer` (or the `vt_patch_bridge` example) when you want to update only changed regions.
Set an origin offset when your editable region starts after a prompt.
In IRIS, Direct mode and SQL shell are sequential `READ` loops, so keep one renderer for the
active `READ`, switch grammar from your `$ZU()` mode signal, and call `clear_state()` when a new
`READ` starts.
`vt_patch_bridge` can auto-select mode:
- with `--origin-row`: incremental viewport diff (`IncrementalRenderer`)
- without `--origin-row` + single-line snapshots: stream-line diff (`StreamLineRenderer`)
- without `--origin-row` + multiline snapshots: full-rerender fallback with relative clear/reposition
For a full end-to-end walkthrough, see `docs/incremental-terminal-highlighting.md`.

```bash
# old and new SQL command snapshots from your IRIS terminal host
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

# emit VT patch from old -> new
cargo run -p render-ansi --example vt_patch_bridge -- \
  /tmp/iris-new.sql tokyonight-dark sql \
  --prev /tmp/iris-old.sql \
  --width 120 --height 40 \
  --origin-row 4 --origin-col 7
```

Output is a patch stream (cursor movement + SGR + erase), not a full-frame redraw.
Patch columns are display-width based (grapheme-aware), so wide Unicode and tabs do not use raw-byte offsets.

If you multiplex independent PTYs/connections in one host process, keep a
`HashMap<String, IncrementalRenderer>` so each terminal ID keeps isolated prior-frame state.

If terminal width/wrap cannot be trusted, use `StreamLineRenderer` for line-local
relative updates (no absolute XY), or switch to full-frame rerender with
`highlight_to_ansi(...)`.

If you use `vt_patch_bridge` without `--origin-row`, the bridge now picks this automatically:
- single-line -> stream-line relative patches
- multiline -> full-rerender fallback patch

## 2) With a native C engine (no ANSI renderer)

### Step 1: Add dependencies

```toml
[dependencies]
highlight-spans = { path = "../syntax-color-objectscript/crates/highlight-spans" }
theme-engine = { path = "../syntax-color-objectscript/crates/theme-engine" }
```

### Step 2: Build spans and resolve styles

```rust
use highlight_spans::{Grammar, SpanHighlighter};
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = b"set x = 42";
    let mut highlighter = SpanHighlighter::new()?;
    let result = highlighter.highlight(source, Grammar::ObjectScript)?;
    let theme = load_theme("tokyonight-dark")?;

    for span in &result.spans {
        let capture = &result.attrs[span.attr_id].capture_name;
        let style = theme.resolve(capture).copied();
        let text = &source[span.start_byte..span.end_byte];

        // pass to your C engine:
        // paint_range(span.start_byte, span.end_byte, style, text);
    }

    Ok(())
}
```

### Step 3: Map style fields into your C paint API

Map `Style` fields to your engine attributes:
- `fg`, `bg`
- `bold`, `italic`, `underline`

### Step 4: Convert byte ranges if needed

If your engine is row/column-based, convert byte offsets to row/column positions before painting.

### Zedit-style bridge example (`attr,start,end` -> C paint ops)

This pattern keeps `theme-engine` separate and prepares a C-friendly operation list that your paint engine can consume.

```rust
use highlight_spans::{Grammar, SpanHighlighter};
use theme_engine::{load_theme, Rgb, Style, Theme};

const FLAG_BOLD: u8 = 0b0001;
const FLAG_ITALIC: u8 = 0b0010;
const FLAG_UNDERLINE: u8 = 0b0100;
const FLAG_HAS_BG: u8 = 0b1000;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CPaintOp {
    pub start_byte: u32,
    pub end_byte: u32,
    pub fg_r: u8,
    pub fg_g: u8,
    pub fg_b: u8,
    pub bg_r: u8,
    pub bg_g: u8,
    pub bg_b: u8,
    pub flags: u8,
}

fn merge_with_normal(style: Option<Style>, normal: Style) -> Style {
    let style = style.unwrap_or_default();
    Style {
        fg: style.fg.or(normal.fg),
        bg: style.bg.or(normal.bg),
        bold: style.bold || normal.bold,
        italic: style.italic || normal.italic,
        underline: style.underline || normal.underline,
    }
}

fn style_to_c_op(span_start: usize, span_end: usize, style: Style) -> Option<CPaintOp> {
    let start_byte = u32::try_from(span_start).ok()?;
    let end_byte = u32::try_from(span_end).ok()?;

    let fg = style.fg.unwrap_or(Rgb::new(220, 220, 220));
    let bg = style.bg.unwrap_or(Rgb::new(0, 0, 0));

    let mut flags = 0u8;
    if style.bold {
        flags |= FLAG_BOLD;
    }
    if style.italic {
        flags |= FLAG_ITALIC;
    }
    if style.underline {
        flags |= FLAG_UNDERLINE;
    }
    if style.bg.is_some() {
        flags |= FLAG_HAS_BG;
    }

    Some(CPaintOp {
        start_byte,
        end_byte,
        fg_r: fg.r,
        fg_g: fg.g,
        fg_b: fg.b,
        bg_r: bg.r,
        bg_g: bg.g,
        bg_b: bg.b,
        flags,
    })
}

pub fn build_c_paint_ops(
    source: &[u8],
    grammar: Grammar,
    theme: &Theme,
    highlighter: &mut SpanHighlighter,
) -> Result<Vec<CPaintOp>, Box<dyn std::error::Error>> {
    let result = highlighter.highlight(source, grammar)?;

    let normal = theme
        .resolve("normal")
        .copied()
        .unwrap_or(Style {
            fg: Some(Rgb::new(220, 220, 220)),
            ..Style::default()
        });

    let mut ops = Vec::with_capacity(result.spans.len());
    for span in &result.spans {
        let capture = result
            .attrs
            .get(span.attr_id)
            .map(|a| a.capture_name.as_str())
            .unwrap_or("normal");

        let resolved = merge_with_normal(theme.resolve(capture).copied(), normal);
        if let Some(op) = style_to_c_op(span.start_byte, span.end_byte, resolved) {
            ops.push(op);
        }
    }

    Ok(ops)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = br#"
Class Demo.Paint
{
  ClassMethod Main() {
    set x = 42  // numeric + comment
  }
}
"#;

    let theme = load_theme("tokyonight-dark")?;
    let mut highlighter = SpanHighlighter::new()?;
    let ops = build_c_paint_ops(source, Grammar::ObjectScript, &theme, &mut highlighter)?;

    for op in &ops {
        // Replace with your C engine bridge call, for example:
        // zedit_paint_range(op.start_byte, op.end_byte, op.fg_r, op.fg_g, op.fg_b, op.bg_r, op.bg_g, op.bg_b, op.flags);
        println!("{op:?}");
    }

    Ok(())
}
```

The key point is that `attr_id` is translated to a capture name via `result.attrs[attr_id]`, then resolved by `theme-engine` into concrete style values before calling your painter.

### Runnable example in this repo

Source:

- `crates/render-ansi/examples/zedit_bridge.rs`
- `crates/render-ansi/examples/vt_patch_bridge.rs`

Run:

```bash
cargo run -p render-ansi --example zedit_bridge -- /path/to/file.cls
```

Optional args:

```bash
cargo run -p render-ansi --example zedit_bridge -- /path/to/file.cls solarized-dark objectscript
```

Output format (one line per paint op):

`start end fg_r fg_g fg_b bg_r bg_g bg_b flags`

For VT patch output instead of paint ops:

```bash
cargo run -p render-ansi --example vt_patch_bridge -- \
  /path/to/file.sql tokyonight-dark sql \
  --origin-row 4 --origin-col 7
```
