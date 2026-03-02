# highlight-spans

`highlight-spans` parses ObjectScript source with Tree-sitter and returns semantic highlight ranges as `(attr_id, start_byte, end_byte)` spans plus an attribute table.

## What You Get

- `SpanHighlighter` for highlighting source bytes or line slices.
- `HighlightResult { attrs, spans }` where:
  - `attrs` maps `attr_id -> capture_name`
  - `spans` contains byte ranges tagged by `attr_id`
- `Grammar` variants:
  - `ObjectScript`
  - `ObjectScriptCore`
  - `ObjectScriptExpr`

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
