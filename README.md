# syntax-color-objectscript

Workspace for syntax coloring with a clean split between:

- syntax analysis (`highlight-spans`)
- style selection (`theme-engine`)

This lets one parser/theme pipeline support multiple outputs:

- ANSI/VT terminal rendering
- native C paint engines
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
    themes/         # embedded JSON theme assets
  render-ansi/      # styled ranges -> ANSI/VT escape output
  theme-engine-ffi/ # C ABI wrapper around theme-engine, highlight-spans, and render-ansi
examples/           # sample source inputs used during highlighting work
```

## Crates

### `highlight-spans`

Purpose:

- Convert source text (ObjectScript class/routine, SQL, Python, Markdown, MDX, XML, JSON, YAML) into `(attr_id, start_byte, end_byte)` spans.
- Return an attribute table mapping `attr_id -> capture_name`.
- Treat `mdx` as a temporary alias to SQL highlighting (for InterSystems MDX content).
- Highlight XML host documents with ObjectScript injection inside `<Implementation>` content.
- [Crate README](crates/highlight-spans/README.md)

Depends on:

- `tree-sitter-objectscript-playground = "1.9.4"`
- `tree-sitter-objectscript-routine = "1.9.4"`
- `tree-sitter-python = "0.25.0"`
- `tree-sitter-md = "0.5.3"`
- `tree-sitter-json = "0.24.8"`
- `tree-sitter-yaml = "0.7.2"`
- `tree-sitter-xml = "0.7.0"`
- `tree-sitter-highlight = ">=0.26.6"`
- `tree-sitter = ">=0.26.6"`
- bundled SQL grammar from `DerekStride/tree-sitter-sql` (`vendor/tree-sitter-sql/src/*`, `vendor/tree-sitter-sql/queries/highlights.scm`)

### `theme-engine`

Purpose:

- Resolve capture names to concrete styles:
  - `fg`/`bg` RGB
  - `bold`, `italic`, `underline`
- Resolve UI-role styles (`statusline`, `tab_active`, `selection`, etc.).
- Provide theme default terminal fg/bg for OSC 10/11 integration.
- Normalize capture keys (`@comment` and `comment` both resolve).
- Support fallback (`comment.documentation -> comment -> normal`).
- Include built-in themes:
  - Tokyo Night: `tokyonight-night`, `tokyonight-storm`, `tokyonight-moon`, `tokyonight-day`
  - Catppuccin: `catppuccin-latte`, `catppuccin-frappe`, `catppuccin-macchiato`, `catppuccin-mocha`
  - Studio: `studio-default`, `aviel`
  - Solarized: `solarized-dark`, `solarized-light`
- [Crate README](crates/theme-engine/README.md)

### `render-ansi`

Purpose:

- Convert highlighted byte spans into ANSI/VT escaped text.
- Provide line-oriented APIs (`Vec<String>`) for terminal rendering.
- Provide incremental VT patching with configurable terminal origin offsets (`row`, `col`).
- Provide bridge-friendly auto mode selection (`vt_patch_bridge`) when origin is omitted.
- Compute incremental patch columns using grapheme/display-width logic (wide Unicode and tabs).
- Provide OSC helpers for terminal default fg/bg updates from theme values.
- Keep renderer logic separate from parsing and theme selection.
- [Crate README](crates/render-ansi/README.md)

### `theme-engine-ffi`

Purpose:

- Expose the full syntax-color pipeline to C hosts via a stable FFI layer.
- Load themes, run highlighting, and render ANSI/VT output from C.
- Provide reusable incremental and single-line patch renderers for terminal UIs.
- Preserve the existing theme-only entry points for style lookup and terminal default colors.
- [Crate README](crates/theme-engine-ffi/README.md)

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

Primary documentation currently lives in the crate READMEs:

- [highlight-spans](crates/highlight-spans/README.md)
- [theme-engine](crates/theme-engine/README.md)
- [render-ansi](crates/render-ansi/README.md)
- [theme-engine-ffi](crates/theme-engine-ffi/README.md)
- [examples/](examples/)

## Test

```bash
cargo test
```
