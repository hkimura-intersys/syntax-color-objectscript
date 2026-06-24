# highlight-spans

`highlight-spans` parses source with Tree-sitter and returns semantic highlight ranges as `(attr_id, start_byte, end_byte)` spans plus an attribute table.

## What You Get

- `SpanHighlighter` for highlighting source bytes or line slices.
- `HighlightResult { attrs, spans }` where:
  - `attrs` maps `attr_id -> capture_name`
  - `spans` contains byte ranges tagged by `attr_id`
- `Grammar` variants:
  - `ObjectScript`
  - `ObjectScriptRoutine`
  - `Sql` (using vendored `DerekStride/tree-sitter-sql` grammar/query assets)
  - `Python`
  - `Markdown` (using `tree-sitter-md` block+inline grammar/query constants)
  - `Mdx` (temporary fallback: uses SQL highlighting)
  - `Xml` (XML host highlighting with ObjectScript injection in `<Implementation>` content)
  - `Json`
  - `Yaml`

## Quick Example

```rust
use highlight_spans::{Grammar, SpanHighlighter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut highlighter = SpanHighlighter::new()?;
    let source = b"set x = 42";
    let result = highlighter.highlight(source, Grammar::ObjectScript)?;

    for span in &result.spans {
        let capture = &result.attrs[span.attr_id].capture_name;
        println!("{capture} {}..{}", span.start_byte, span.end_byte);
    }

    Ok(())
}
```

## Typical Next Step

Use `theme-engine` to resolve `capture_name` into styles, then pass styled ranges to a renderer (for example `render-ansi`).

## Timing Logs

Highlight timing logs are off by default. You can enable them either per highlighter:

```rust
let mut highlighter = SpanHighlighter::new()?.with_timing_logs_enabled(true);
```

or process-wide for newly created highlighters:

```bash
SYNTAX_COLOR_LOG_HIGHLIGHT_TIMINGS=1 cargo run ...
```

When enabled, `highlight-spans` writes one stderr line per highlight call with the grammar, source size, span count, and a small timing breakdown for the base pass and injection work.
