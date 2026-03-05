# syntax-color-objectscript

Workspace for syntax coloring with a clean split between:

- syntax analysis (`highlight-spans`)
- style selection (`theme-engine`)

This lets one parser/theme pipeline support multiple outputs:

- ANSI/VT terminal rendering
- native C/TUI paint engines
- future GUI/web adapters

## What This Accomplishes

This project turns source code into style-ready data in two stages:

1. Parse and classify code tokens into semantic capture names (`keyword`, `comment`, `number`, etc.).
2. Map those names to concrete visual styles (RGB + bold/italic/underline).

The key benefit is decoupling parser logic from theme logic, so each can evolve independently.

## Workspace Layout

```text
crates/
  highlight-spans/  # Tree-sitter -> spans + attr table
  theme-engine/     # capture name -> style resolution + built-in theme loader
    themes/         # built-in JSON themes (tokyonight/solarized)
  render-ansi/      # styled ranges -> ANSI/VT escape output
docs/
  architecture.md
  highlight-spans.md
  theme-engine.md
  render-ansi.md
  integration.md
```

## Crates

### `highlight-spans`

Purpose:

- Convert source text (ObjectScript/SQL/Python/Markdown/MDX) into `(attr_id, start_byte, end_byte)` spans.
- Return an attribute table mapping `attr_id -> capture_name`.
- Treat `mdx` as a temporary alias to SQL highlighting (for InterSystems MDX content).
- [Crate README](crates/highlight-spans/README.md)

Depends on:

- `tree-sitter-objectscript = "1.4.3"`
- `tree-sitter-python = "0.25.0"`
- `tree-sitter-md = "0.5.3"`
- `tree-sitter-highlight = ">=0.26.6"`
- `tree-sitter = ">=0.26.6"`
- bundled SQL grammar from `DerekStride/tree-sitter-sql` (`vendor/tree-sitter-sql/src/*`, `vendor/tree-sitter-sql/queries/highlights.scm`)

### `theme-engine`

Purpose:

- Resolve capture names to concrete styles:
  - `fg`/`bg` RGB
  - `bold`, `italic`, `underline`
- Normalize capture keys (`@comment` and `comment` both resolve).
- Support fallback (`comment.documentation -> comment -> normal`).
- Include built-in themes: `tokyonight-dark`, `tokyonight-moon`, `tokyonight-light`, `tokyonight-day`, `solarized-dark`, `solarized-light`.
- [Crate README](crates/theme-engine/README.md)

### `render-ansi`

Purpose:

- Convert highlighted byte spans into ANSI/VT escaped text.
- Provide line-oriented APIs (`Vec<String>`) for terminal rendering.
- Provide incremental VT patching with configurable terminal origin offsets (`row`, `col`).
- Compute incremental patch columns using grapheme/display-width logic (wide Unicode and tabs).
- Keep renderer logic separate from parsing and theme selection.
- [Crate README](crates/render-ansi/README.md)

## Data Flow

```text
source code
  -> highlight-spans
     -> attrs: [{id, capture_name}]
     -> spans: [{attr_id, start_byte, end_byte}]
  -> theme-engine
     -> style per capture_name
  -> renderer (ANSI or C painter)
```

## Quick Example

```rust
use highlight_spans::{Grammar, SpanHighlighter};
use theme_engine::Theme;

let mut highlighter = SpanHighlighter::new()?;
let result = highlighter.highlight(b"set x = 42", Grammar::ObjectScript)?;

let theme = Theme::from_json_str(r#"{
  "styles": {
    "normal": { "fg": { "r": 220, "g": 220, "b": 220 } },
    "number": { "fg": { "r": 255, "g": 180, "b": 120 } }
  }
}"#)?;

for span in &result.spans {
    let capture = &result.attrs[span.attr_id].capture_name;
    let style = theme.resolve(capture);
    // renderer applies `style` to source[span.start_byte..span.end_byte]
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Documentation

- [arc42 Architecture](docs/arc42.md)
- [C4 Model](docs/c4.md)
- [Design Doc](docs/design-doc.md)
- [Integration Steps](docs/integration.md)
- [Incremental Terminal Highlighting Guide](docs/incremental-terminal-highlighting.md)
- [Release and Publish Guide](docs/release.md)
- [Usage Examples](docs/usage-examples.md)
- [Data Structure: HighlightResult](docs/data-structures/highlight-result.md)
- [Data Structure: Theme](docs/data-structures/theme.md)
- [Data Structure: StyledSpan](docs/data-structures/styled-span.md)
- [Code Health Report](docs/code-health-report.md)
- [Edge-Case Test Plan](test-plan.md)

## Test

```bash
cargo test
```
