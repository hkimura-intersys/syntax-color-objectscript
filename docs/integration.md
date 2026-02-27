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

Use `highlight_lines_to_ansi_lines(...)` when your terminal UI redraws line-by-line.

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
