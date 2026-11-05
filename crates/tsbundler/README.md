# tsbundler

`tsbundler` is the C API used by the zedit editor to get syntax highlighting from this workspace. It wraps `highlight-spans`, `theme-engine`, and `render-ansi` behind a small context/document interface and fills zedit's per-character `zedit_attr_t` buffers.

The crate builds as a shared library named **`irisutils`**:

| Platform | Artifact |
| --- | --- |
| macOS | `libirisutils.dylib` |
| Linux | `libirisutils.so` |
| Windows | `irisutils.dll` + `irisutils.dll.lib` |

Prebuilt binaries live in the repository's top-level `tsbundler/` directory and are regenerated with `rebuild-release.sh`.

The C header is [`include/zedit_ts.h`](../../include/zedit_ts.h). Include it after `zedit.h`, which defines `ZEDIT_TYPES_DEFINED`, `ZEDIT_SYN_ELEMENT_DEFINED` and `ZEDIT_RGBNONE`; without `zedit.h`, the header supplies its own minimal types and constants, which is handy for testing. zedit and the IRIS kernel keep their own copies of this header, so copy it there after changing it.

## API Overview

Context:

- `tsbundler_init()` / `tsbundler_free(ctx)`: create and destroy a context. The context loads every built-in theme up front.
- `tsbundler_set_logger(ctx, fn, payload)`: receive parse/info log messages.

Languages:

- `tsbundler_enum_langs(ctx, callback, userdata)`: enumerate bundled languages with their file extensions. Pass `callback = NULL` to get only the count.
- `tsbundler_supports_lang(ctx, lang)`: returns non-zero if `lang` is accepted. This takes any name or alias that `Grammar::from_name` accepts (for example `cls`, `mac`, `py`, `diff`), including grammars that `tsbundler_enum_langs` doesn't list (such as `markdown_inline`).

Themes:

- `tsbundler_enum_themes(ctx, callback, userdata)`: enumerate built-in theme names.
- `tsbundler_theme_activate(ctx, name, callback, userdata)`: make a theme active and report each syntax element (`"@keyword"`, fg, bg, attrs, index). The `index` is the dense, 1-based `zedit_attr_t` value that appears in attribute buffers; `0` means no attribute. Returns the element count, or a negative `tsbundler_error` value.
- `tsbundler_theme_resolve_ui(ctx, role, &fg, &bg, &attrs)`: resolve a UI role (`default_fg`, `default_bg`, `statusline`, `statusline_inactive`, `tab_active`, `tab_inactive`, `selection`, `cursorline`) from the active theme.

Colors are packed `0x00RRGGBB`; `ZEDIT_RGBNONE` (`0xFFFFFFFF`) means "use the default".

Documents:

- `tsbundler_doc_create(ctx, lang)` / `tsbundler_doc_free(doc)`
- `tsbundler_doc_parse_full(doc, read, read_payload, encoding, total_lines, attrs, attrs_payload)`: read the whole document through a `TSInputRead`-style callback (UTF-8, UTF-16LE, or UTF-16BE), highlight it, then call `attrs(payload, row, nchars)` for each line. zedit returns its own `zedit_attr_t[nchars]` buffer to fill, or `NULL` to skip the line.
- `tsbundler_doc_edit(doc, edits, nedits, ..., on_changed, on_changed_payload)`: re-highlight after an edit and call `on_changed(start_line, end_line)` only for lines whose attributes changed.

A theme must be active before parsing; otherwise the document calls return `TSBUNDLER_ERR_NOTHEME`.

IRIS command-line recall:

- `recall_ansi_color_render_range(srcbuf, linelen, start, n, outbuf, outcap, lang_tag, &len)`: render part of one 8-bit line as ANSI-colored text for the IRIS terminal (`termio.c`). It needs no context or active theme; it keeps its own highlighter and always uses `tokyonight-dark`. `lang_tag` is 1 for SQL, 2 for Python, 3 for MDX, and anything else for ObjectScript. If `outcap` is too small, it returns 1 with `len` set to the size needed and writes nothing.

## Typical Usage

```c
#include "zedit_ts.h"

tsbundler_ctx ctx = tsbundler_init();
tsbundler_theme_activate(ctx, "tokyonight-dark", on_element, NULL);

tsbundler_doc doc = tsbundler_doc_create(ctx, "objectscript_udl");
tsbundler_doc_parse_full(doc, read_text, buffer, TSBUNDLER_ENC_UTF16LE,
                         line_count, line_attrs, buffer);

/* after the user edits the buffer */
tsbundler_doc_edit(doc, &edit, 1, read_text, buffer, TSBUNDLER_ENC_UTF16LE,
                   line_count, line_attrs, buffer, repaint_lines, view);

tsbundler_doc_free(doc);
tsbundler_free(ctx);
```

## Notes and Limitations

- `tsbundler_doc_edit` accepts edit descriptors, but the current implementation does a full reparse and then diffs attributes line by line. The descriptors are reserved for future incremental tree reuse.
- Attribute buffers carry one `zedit_attr_t` per character, so a span's foreground and background come from a single theme element. The ambient diff background that `highlight-spans` attaches to injected spans (`background_attr_id`) isn't represented.
