# highlight-spans

`highlight-spans` parses source with Tree-sitter and returns semantic highlight ranges as `(attr_id, start_byte, end_byte)` spans plus an attribute table.

## What You Get

- `SpanHighlighter` for highlighting source bytes or line slices.
- `HighlightResult { attrs, spans }` where:
  - `attrs` maps `attr_id -> capture_name`
  - `spans` contains byte ranges tagged by `attr_id`, plus an optional `background_attr_id` (see [Diff Highlighting](#diff-highlighting))
- `Grammar` variants:
  - `ObjectScript` (playground grammar)
  - `ObjectScriptRoutine` (`.mac`, `.int`, `.inc`, `.rtn`)
  - `ObjectScriptUdl` (UDL class grammar from `tree-sitter-objectscript`)
  - `Sql` (using vendored `DerekStride/tree-sitter-sql` grammar/query assets)
  - `Python`
  - `Markdown` (using `tree-sitter-md` block grammar; inline content is injected as `MarkdownInline`)
  - `MarkdownInline` (emphasis, links, code spans; normally reached through `Markdown` injection)
  - `Mdx` (temporary fallback: uses SQL highlighting)
  - `Xml` (XML host highlighting with ObjectScript injection in `<Implementation>` content)
  - `Json`
  - `Yaml`
  - `Css`
  - `Html`
  - `JavaScript`
  - `JsDoc`
  - `Regex`
  - `Toml`
  - `Diff` (unified diffs, with per-file language injection)
  - `Rust` (`.rs`)
- Name lookup helpers:
  - `Grammar::from_name("py")` parses canonical names and aliases (`cls`, `mac`, `rtn`, `udl`, `yml`, `js`, `patch`, `rs`, ...)
  - `Grammar::from_filename("src/foo.py")` derives a grammar from a file extension
  - `Grammar::canonical_name()` and `Grammar::supported_names()`

## Quick Example

```rust
use highlight_spans::highlight_structures::{Grammar, SpanHighlighter};

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

## Injections

Every grammar that ships an injection query has it applied: embedded SQL, HTML, JavaScript, XML and
Python in ObjectScript classes and routines, `<script>`/`<style>` in HTML, fenced code blocks, HTML and
inline content in Markdown, JSDoc and regex in JavaScript, `<Implementation>` bodies in XML, and macro
invocation and `macro_rules!` bodies in Rust (highlighted as Rust).

Each source is parsed once and the same tree feeds both the highlight and injection queries. Injected
regions are parsed with their own grammar and run through the full pipeline, so their injections apply
too, up to three levels deep (for example Markdown -> HTML -> JavaScript). Regions naming a language
with no matching grammar (such as `ispl` or `latex`) keep the host's highlighting.

## Diff Highlighting

`Grammar::Diff` highlights unified diffs with `tree-sitter-diff` and injects the changed file's
own grammar into hunk lines. The injected language comes from the `+++`/`---` filename via
`Grammar::from_filename`; files with an unrecognized extension keep plain diff highlighting.

Diff injection needs a git-style file block that starts with a `diff --git a/... b/...` line. Bare
`---`/`+++` headers (plain `diff -u` output) don't parse as a file block, so their hunks keep plain diff
highlighting. An `index` line is optional, but a malformed one also prevents the block from parsing.

Injected spans inside added/removed lines carry `background_attr_id` pointing at the diff
classification (for example `diff.plus`). Renderers should use only the *background* of that
attribute's style, layered under the token's own style, so a Python `keyword` inside a `+` line
keeps its foreground color and still gets the diff's added-line background. `render-ansi`
does this automatically. With nested injections the outermost background wins, so inline Markdown
inside a `.md` file's `+` line still sits on `diff.plus`.

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
