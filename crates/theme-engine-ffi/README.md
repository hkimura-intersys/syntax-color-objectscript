# theme-engine-ffi

`theme-engine-ffi` now exposes the full syntax-color pipeline to C hosts:

- `theme-engine` theme loading and style resolution
- `highlight-spans` token highlighting
- `render-ansi` full-frame ANSI rendering
- `render-ansi` incremental and stream-line VT patch rendering

It supports:

- loading built-in themes (`theme_engine_theme_load_builtin`)
- loading JSON themes (`theme_engine_theme_load_json`)
- resolving syntax capture styles (`theme_engine_theme_resolve_capture`)
- resolving UI role styles (`theme_engine_theme_resolve_ui`)
- reading default terminal fg/bg (`theme_engine_theme_default_terminal_colors`)
- creating reusable highlighters (`syntax_color_highlighter_new`)
- returning highlight attrs/spans (`syntax_color_highlighter_highlight`)
- resolving render-ready styled spans (`syntax_color_resolve_styled_spans`)
- rendering ANSI output (`syntax_color_render_ansi`, `syntax_color_highlight_to_ansi`)
- rendering per-line ANSI output (`syntax_color_render_ansi_lines`, `syntax_color_highlight_to_ansi_lines`)
- generating incremental VT patches (`syntax_color_incremental_renderer_*`)
- generating single-line relative VT patches (`syntax_color_stream_line_renderer_*`)
- emitting OSC default-color helpers (`syntax_color_osc_*`)

See [`include/theme_engine_ffi.h`](include/theme_engine_ffi.h) for the C ABI.

Functions that take a `grammar` argument expect a `SyntaxColorGrammar` value. Every grammar `highlight-spans` supports except `Rust` has a code (0–17); `Rust` has no code yet, so C callers of this ABI can't select it. existing codes never change and new grammars are appended, so values compiled into older C callers stay valid.

Limitations:

- `SyntaxColorSpan` has no `background_attr_id` field, so the ambient diff background that
  `highlight-spans` attaches to injected spans (see the `highlight-spans` README) is not exposed
  over this ABI. Spans passed back in from C are treated as having no ambient background.

For the zedit editor integration, see the separate [`tsbundler`](../tsbundler/README.md) crate.
