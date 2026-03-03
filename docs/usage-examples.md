# Usage Example Catalog

## Example UX-001: Generate Highlight Spans from ObjectScript Source

**Audience:** Developers using `highlight-spans` directly.
**Topic:** How to extract semantic spans and capture names.
**Goal:** Convert ObjectScript bytes into `HighlightResult` for downstream rendering.

### Context

- When to use it: You need parser output without choosing a render format yet.
- When not to use it: You only need final ANSI strings; use `render-ansi` orchestration helpers instead.

### Prerequisites

- Dependencies: `highlight-spans` crate.
- Config: None.
- Permissions: None.

### Example (Minimal)

**Introduction:** This example shows how to parse source and inspect captured spans.

```rust
use highlight_spans::{Grammar, SpanHighlighter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut highlighter = SpanHighlighter::new()?;
    let source = b"set x = 42";
    let result = highlighter.highlight(source, Grammar::ObjectScript)?;

    for span in &result.spans {
        let capture = &result.attrs[span.attr_id].capture_name;
        println!("{} {}..{}", capture, span.start_byte, span.end_byte);
    }

    Ok(())
}
```

**Explanation:**
1. `SpanHighlighter::new` initializes the ObjectScript highlight configuration.
2. `highlight` returns `HighlightResult` with attr table and byte ranges.
3. `attr_id` indexes into `attrs` to recover capture names.

### Example (Advanced)

**Introduction:** This example highlights line-based input while preserving the same result type.

```rust
use highlight_spans::{Grammar, SpanHighlighter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut highlighter = SpanHighlighter::new()?;
    let lines = vec!["set x = 1", "set y = 2"];
    let result = highlighter.highlight_lines(&lines, Grammar::ObjectScript)?;

    assert!(!result.spans.is_empty());
    Ok(())
}
```

**Explanation:**
1. `highlight_lines` joins line slices and delegates to `highlight`.
2. The returned structure is still `HighlightResult`.
3. You can reuse the same downstream renderer pipeline.

### Evidence

- `crates/highlight-spans/src/lib.rs:49` (constructor)
- `crates/highlight-spans/src/lib.rs:63` (highlight API)
- `crates/highlight-spans/src/lib.rs:112` (line API)
- `crates/highlight-spans/src/lib.rs:30` (result type)

### Validation Notes

- Verified signature against code: Yes
- Verified usage in tests or examples: Yes (`crates/highlight-spans/src/lib.rs:161`)
- Mismatches or assumptions: None

## Example UX-002: Build and Resolve Themes with Fallback

**Audience:** Developers using `theme-engine`.
**Topic:** How to load/construct themes and resolve capture names safely.
**Goal:** Retrieve consistent `Style` values for parser captures.

### Context

- When to use it: You need stable style resolution across capture-name variants.
- When not to use it: You only need built-in theme names and no custom mapping.

### Prerequisites

- Dependencies: `theme-engine` crate.
- Config: Theme JSON/TOML text or built-in theme name.
- Permissions: None.

### Example (Minimal)

**Introduction:** This example parses JSON theme content and resolves a style.

```rust
use theme_engine::Theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = Theme::from_json_str(r#"
    {
      "styles": {
        "normal": { "fg": { "r": 200, "g": 200, "b": 200 } },
        "comment": { "italic": true }
      }
    }
    "#)?;

    let style = theme.resolve("@comment.documentation").unwrap();
    assert!(style.italic);
    Ok(())
}
```

**Explanation:**
1. `from_json_str` parses wrapped style documents.
2. `resolve` normalizes `@` prefix and case.
3. Dotted fallback maps `comment.documentation` to `comment`.

### Example (Advanced)

**Introduction:** This example loads a built-in theme alias and falls back to `normal` for unknown keys.

```rust
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = load_theme("tokyo-night")?;
    let style = theme.resolve("unknown.capture");
    assert!(style.is_some());
    Ok(())
}
```

**Explanation:**
1. `load_theme` accepts canonical names and aliases.
2. Unknown captures fall back to `normal` when present.
3. Built-in themes include `normal` (validated in tests).

### Evidence

- `crates/theme-engine/src/lib.rs:134` (JSON loader)
- `crates/theme-engine/src/lib.rs:117` (resolve behavior)
- `crates/theme-engine/src/lib.rs:125` (dotted fallback)
- `crates/theme-engine/src/lib.rs:160` (built-in loader)
- `crates/theme-engine/src/lib.rs:298` (alias test)

### Validation Notes

- Verified signature against code: Yes
- Verified usage in tests or examples: Yes (`crates/theme-engine/src/lib.rs:240`, `crates/theme-engine/src/lib.rs:298`)
- Mismatches or assumptions: Assumes `normal` exists in selected theme (`crates/theme-engine/src/lib.rs:131`)

## Example UX-003: End-to-End ANSI Rendering Pipeline

**Audience:** Developers integrating highlighting directly into terminal output.
**Topic:** How to run parse + theme + render in one call.
**Goal:** Produce ANSI-escaped strings from source input.

### Context

- When to use it: You want a single API that orchestrates all stages.
- When not to use it: You need custom rendering behavior beyond ANSI output.

### Prerequisites

- Dependencies: `render-ansi`, `theme-engine`.
- Config: Selected `Grammar` and a `Theme`.
- Permissions: None.

### Example (Minimal)

**Introduction:** This example renders a full source buffer to ANSI.

```rust
use highlight_spans::Grammar;
use render_ansi::highlight_to_ansi;
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = load_theme("tokyonight-dark")?;
    let out = highlight_to_ansi(b"set x = 42", Grammar::ObjectScript, &theme)?;
    println!("{}", out);
    Ok(())
}
```

**Explanation:**
1. `highlight_to_ansi` creates a highlighter and runs the full pipeline.
2. Spans are converted to `StyledSpan` entries with resolved style.
3. Renderer emits SGR sequences plus reset markers.

### Example (Advanced)

**Introduction:** This example renders per-line output for incremental terminal redraws.

```rust
use highlight_spans::Grammar;
use render_ansi::highlight_lines_to_ansi_lines;
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = load_theme("tokyo-night")?;
    let lines = vec!["set x = 1", "set y = 2"];
    let rendered = highlight_lines_to_ansi_lines(&lines, Grammar::ObjectScript, &theme)?;

    assert_eq!(rendered.len(), 2);
    Ok(())
}
```

**Explanation:**
1. Line API returns `Vec<String>` aligned to input line count.
2. Multi-line styled spans are clipped per line in renderer internals.
3. This is useful for TUI panes that redraw line-by-line.

### Evidence

- `crates/render-ansi/src/lib.rs:118` (orchestration API)
- `crates/render-ansi/src/lib.rs:133` (highlight call)
- `crates/render-ansi/src/lib.rs:134` (style resolution)
- `crates/render-ansi/src/lib.rs:138` (line API)
- `crates/render-ansi/src/lib.rs:98` (line clipping)

### Validation Notes

- Verified signature against code: Yes
- Verified usage in tests or examples: Yes (`crates/render-ansi/src/lib.rs:300`, `crates/render-ansi/src/lib.rs:318`)
- Mismatches or assumptions: None

## Doc/Code Mismatches

- No mismatches found in the documented examples versus current source/test definitions.
