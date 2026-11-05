use crate::c_structures::*;
use highlight_spans::highlight_structures::{
    Attr, Grammar, HighlightResult, Span, SpanHighlighter,
};
use render_ansi::ansi_structures::{ColorMode, StyledSpan};
use render_ansi::common::{
    highlight_to_ansi as render_highlight_to_ansi, osc_reset_default_background,
    osc_reset_default_colors, osc_reset_default_foreground, osc_set_default_colors, render_ansi,
    render_ansi_lines, resolve_styled_spans, resolve_styled_spans_for_source,
};
use std::ffi::{CStr, c_char};
use std::ptr;
use theme_engine::common::load_theme;
use theme_engine::theme_structures::{Rgb, Style, Theme};

fn rgb_to_ffi(rgb: Rgb) -> ThemeEngineRgb {
    ThemeEngineRgb {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
    }
}

fn rgb_from_ffi(rgb: ThemeEngineRgb) -> Rgb {
    Rgb {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
    }
}

fn style_to_ffi(style: Style) -> ThemeEngineStyle {
    let (has_fg, fg) = if let Some(color) = style.fg {
        (1, rgb_to_ffi(color))
    } else {
        (0, ThemeEngineRgb::default())
    };
    let (has_bg, bg) = if let Some(color) = style.bg {
        (1, rgb_to_ffi(color))
    } else {
        (0, ThemeEngineRgb::default())
    };

    ThemeEngineStyle {
        has_fg,
        fg,
        has_bg,
        bg,
        bold: u8::from(style.bold),
        italic: u8::from(style.italic),
        underline: u8::from(style.underline),
    }
}

fn style_from_ffi(style: ThemeEngineStyle) -> Style {
    Style {
        fg: (style.has_fg != 0).then(|| rgb_from_ffi(style.fg)),
        bg: (style.has_bg != 0).then(|| rgb_from_ffi(style.bg)),
        bold: style.bold != 0,
        italic: style.italic != 0,
        underline: style.underline != 0,
    }
}

unsafe fn parse_cstr<'a>(value: *const c_char) -> Result<&'a str, i32> {
    if value.is_null() {
        return Err(SYNTAX_COLOR_FFI_ERR_NULL);
    }
    // SAFETY: validated non-null above; caller promises valid C string lifetime.
    let cstr = unsafe { CStr::from_ptr(value) };
    cstr.to_str().map_err(|_| SYNTAX_COLOR_FFI_ERR_UTF8)
}

unsafe fn parse_bytes<'a>(data: *const u8, len: usize) -> Result<&'a [u8], i32> {
    if len == 0 {
        return Ok(&[]);
    }
    if data.is_null() {
        return Err(SYNTAX_COLOR_FFI_ERR_NULL);
    }
    // SAFETY: caller provided a non-null pointer and length pair.
    Ok(unsafe { std::slice::from_raw_parts(data, len) })
}

unsafe fn parse_utf8_bytes<'a>(data: *const u8, len: usize) -> Result<&'a str, i32> {
    let bytes = unsafe { parse_bytes(data, len) }?;
    std::str::from_utf8(bytes).map_err(|_| SYNTAX_COLOR_FFI_ERR_UTF8)
}

unsafe fn parse_ffi_string(value: &SyntaxColorString) -> Result<&str, i32> {
    unsafe { parse_utf8_bytes(value.data.cast_const(), value.len) }
}

/// Maps a C `SyntaxColorGrammar` value to a [`Grammar`].
///
/// Codes are part of the C ABI: existing values must never be renumbered, and
/// new grammars are appended (see `include/theme_engine_ffi.h`).
fn parse_grammar(grammar: i32) -> Result<Grammar, i32> {
    match grammar {
        0 => Ok(Grammar::ObjectScript),
        1 => Ok(Grammar::ObjectScriptRoutine),
        2 => Ok(Grammar::Sql),
        3 => Ok(Grammar::Python),
        4 => Ok(Grammar::Markdown),
        5 => Ok(Grammar::Mdx),
        6 => Ok(Grammar::Xml),
        7 => Ok(Grammar::Json),
        8 => Ok(Grammar::Yaml),
        9 => Ok(Grammar::Css),
        10 => Ok(Grammar::Html),
        11 => Ok(Grammar::JavaScript),
        12 => Ok(Grammar::JsDoc),
        13 => Ok(Grammar::ObjectScriptUdl),
        14 => Ok(Grammar::Regex),
        15 => Ok(Grammar::Toml),
        16 => Ok(Grammar::Diff),
        17 => Ok(Grammar::MarkdownInline),
        18 => Ok(Grammar::Rust),
        _ => Err(SYNTAX_COLOR_FFI_ERR_INVALID_ARGUMENT),
    }
}

fn parse_color_mode(color_mode: i32) -> Result<ColorMode, i32> {
    match color_mode {
        0 => Ok(ColorMode::TrueColor),
        1 => Ok(ColorMode::Ansi256),
        2 => Ok(ColorMode::Ansi16),
        _ => Err(SYNTAX_COLOR_FFI_ERR_INVALID_ARGUMENT),
    }
}

fn bytes_to_ffi(bytes: Vec<u8>) -> SyntaxColorString {
    if bytes.is_empty() {
        return SyntaxColorString::default();
    }

    let len = bytes.len();
    let raw = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;
    SyntaxColorString { data: raw, len }
}

fn string_to_ffi(value: String) -> SyntaxColorString {
    bytes_to_ffi(value.into_bytes())
}

fn strings_to_ffi(values: Vec<String>) -> SyntaxColorStringArray {
    if values.is_empty() {
        return SyntaxColorStringArray::default();
    }

    let items = values.into_iter().map(string_to_ffi).collect::<Vec<_>>();
    let count = items.len();
    let raw = Box::into_raw(items.into_boxed_slice()) as *mut SyntaxColorString;
    SyntaxColorStringArray { items: raw, count }
}

unsafe fn free_boxed_slice<T>(data: *mut T, len: usize) {
    if data.is_null() {
        return;
    }
    // SAFETY: the pointer/length pair originated from Box::into_raw on a boxed slice here.
    let raw = ptr::slice_from_raw_parts_mut(data, len);
    let _ = unsafe { Box::from_raw(raw) };
}

fn highlight_result_to_ffi(result: HighlightResult) -> SyntaxColorHighlightResult {
    let attr_count = result.attrs.len();
    let span_count = result.spans.len();
    let attrs = result
        .attrs
        .into_iter()
        .map(|attr| SyntaxColorAttr {
            id: attr.id,
            capture_name: string_to_ffi(attr.capture_name),
        })
        .collect::<Vec<_>>();
    let spans = result
        .spans
        .into_iter()
        .map(|span| SyntaxColorSpan {
            attr_id: span.attr_id,
            start_byte: span.start_byte,
            end_byte: span.end_byte,
        })
        .collect::<Vec<_>>();

    let (attrs_ptr, attr_count) = if attrs.is_empty() {
        (ptr::null_mut(), 0)
    } else {
        (
            Box::into_raw(attrs.into_boxed_slice()) as *mut SyntaxColorAttr,
            attr_count,
        )
    };
    let (spans_ptr, span_count) = if spans.is_empty() {
        (ptr::null_mut(), 0)
    } else {
        (
            Box::into_raw(spans.into_boxed_slice()) as *mut SyntaxColorSpan,
            span_count,
        )
    };

    SyntaxColorHighlightResult {
        attrs: attrs_ptr,
        attr_count,
        spans: spans_ptr,
        span_count,
    }
}

unsafe fn highlight_result_from_ffi(
    result: *const SyntaxColorHighlightResult,
) -> Result<HighlightResult, i32> {
    if result.is_null() {
        return Err(SYNTAX_COLOR_FFI_ERR_NULL);
    }
    // SAFETY: result non-null checked above.
    let result = unsafe { &*result };
    if result.attr_count > 0 && result.attrs.is_null() {
        return Err(SYNTAX_COLOR_FFI_ERR_NULL);
    }
    if result.span_count > 0 && result.spans.is_null() {
        return Err(SYNTAX_COLOR_FFI_ERR_NULL);
    }

    let attrs = if result.attr_count == 0 {
        Vec::new()
    } else {
        // SAFETY: attr_count/attrs validated above.
        let raw = unsafe { std::slice::from_raw_parts(result.attrs, result.attr_count) };
        let mut attrs = Vec::with_capacity(raw.len());
        for attr in raw {
            let capture_name = unsafe { parse_ffi_string(&attr.capture_name) }?.to_owned();
            attrs.push(Attr {
                id: attr.id,
                capture_name,
            });
        }
        attrs
    };

    let spans = if result.span_count == 0 {
        Vec::new()
    } else {
        // SAFETY: span_count/spans validated above.
        let raw = unsafe { std::slice::from_raw_parts(result.spans, result.span_count) };
        raw.iter()
            .map(|span| Span {
                attr_id: span.attr_id,
                start_byte: span.start_byte,
                end_byte: span.end_byte,
                // The C ABI's RawSpan has no background_attr_id field --
                // ambient-background injection composition is a Rust-side
                // concern only, not yet exposed across this FFI boundary.
                background_attr_id: None,
            })
            .collect::<Vec<_>>()
    };

    Ok(HighlightResult { attrs, spans })
}

fn styled_spans_to_ffi(spans: Vec<StyledSpan>) -> SyntaxColorStyledSpanBuffer {
    if spans.is_empty() {
        return SyntaxColorStyledSpanBuffer::default();
    }

    let converted = spans
        .into_iter()
        .map(|span| {
            let (has_style, style) = if let Some(style) = span.style {
                (1, style_to_ffi(style))
            } else {
                (0, ThemeEngineStyle::default())
            };
            SyntaxColorStyledSpan {
                start_byte: span.start_byte,
                end_byte: span.end_byte,
                has_style,
                style,
            }
        })
        .collect::<Vec<_>>();
    let count = converted.len();
    let raw = Box::into_raw(converted.into_boxed_slice()) as *mut SyntaxColorStyledSpan;
    SyntaxColorStyledSpanBuffer { spans: raw, count }
}

unsafe fn styled_spans_from_ffi(
    spans: *const SyntaxColorStyledSpan,
    count: usize,
) -> Result<Vec<StyledSpan>, i32> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if spans.is_null() {
        return Err(SYNTAX_COLOR_FFI_ERR_NULL);
    }
    // SAFETY: spans non-null checked above and count provided by caller.
    let raw = unsafe { std::slice::from_raw_parts(spans, count) };
    Ok(raw
        .iter()
        .map(|span| StyledSpan {
            start_byte: span.start_byte,
            end_byte: span.end_byte,
            style: (span.has_style != 0).then(|| style_from_ffi(span.style)),
        })
        .collect::<Vec<_>>())
}

unsafe fn theme_ref<'a>(theme: *const ThemeEngineTheme) -> Result<&'a Theme, i32> {
    if theme.is_null() {
        return Err(SYNTAX_COLOR_FFI_ERR_NULL);
    }
    // SAFETY: theme pointer validated above.
    let theme = unsafe { &*theme };
    Ok(&theme.theme)
}

unsafe fn with_optional_highlighter<R>(
    highlighter: *mut SyntaxColorHighlighter,
    f: impl FnOnce(&mut SpanHighlighter) -> Result<R, i32>,
) -> Result<R, i32> {
    if highlighter.is_null() {
        let mut temporary = SpanHighlighter::new().map_err(|_| SYNTAX_COLOR_FFI_ERR_HIGHLIGHT)?;
        return f(&mut temporary);
    }

    // SAFETY: highlighter pointer validated above.
    let highlighter = unsafe { &mut *highlighter };
    f(&mut highlighter.highlighter)
}

unsafe fn renderer_mut<'a>(
    renderer: *mut SyntaxColorIncrementalRenderer,
) -> Result<&'a mut render_ansi::ansi_structures::IncrementalRenderer, i32> {
    if renderer.is_null() {
        return Err(SYNTAX_COLOR_FFI_ERR_NULL);
    }
    // SAFETY: renderer pointer validated above.
    let renderer = unsafe { &mut *renderer };
    Ok(&mut renderer.renderer)
}

unsafe fn stream_renderer_mut<'a>(
    renderer: *mut SyntaxColorStreamLineRenderer,
) -> Result<&'a mut render_ansi::ansi_structures::StreamLineRenderer, i32> {
    if renderer.is_null() {
        return Err(SYNTAX_COLOR_FFI_ERR_NULL);
    }
    // SAFETY: renderer pointer validated above.
    let renderer = unsafe { &mut *renderer };
    Ok(&mut renderer.renderer)
}

fn write_string_output(out: *mut SyntaxColorString, value: String) -> i32 {
    if out.is_null() {
        return SYNTAX_COLOR_FFI_ERR_NULL;
    }
    // SAFETY: out checked non-null above.
    unsafe { *out = string_to_ffi(value) };
    SYNTAX_COLOR_FFI_OK
}

fn write_string_array_output(out: *mut SyntaxColorStringArray, value: Vec<String>) -> i32 {
    if out.is_null() {
        return SYNTAX_COLOR_FFI_ERR_NULL;
    }
    // SAFETY: out checked non-null above.
    unsafe { *out = strings_to_ffi(value) };
    SYNTAX_COLOR_FFI_OK
}

/// Frees a byte buffer previously returned by this library.
///
/// # Safety
///
/// - `value` must be null or point to a `SyntaxColorString` that is either empty (zeroed) or was
///   filled in by this library and not modified since.
/// - The buffer must not be read after this call. The struct is reset to empty, so freeing it again
///   is a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_string_free(value: *mut SyntaxColorString) {
    if value.is_null() {
        return;
    }
    // SAFETY: value pointer validated above.
    let value = unsafe { &mut *value };
    // SAFETY: data/len originate from bytes_to_ffi in this library.
    unsafe { free_boxed_slice(value.data, value.len) };
    *value = SyntaxColorString::default();
}

/// Frees an array of byte buffers previously returned by this library.
///
/// # Safety
///
/// - `values` must be null or point to a `SyntaxColorStringArray` that is either empty (zeroed) or
///   was filled in by this library, with neither the array nor its strings modified or freed
///   individually since.
/// - None of the strings may be read after this call. The struct is reset to empty, so freeing it
///   again is a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_string_array_free(values: *mut SyntaxColorStringArray) {
    if values.is_null() {
        return;
    }
    // SAFETY: values pointer validated above.
    let values = unsafe { &mut *values };
    if !values.items.is_null() {
        // SAFETY: items/count originate from strings_to_ffi in this library.
        let items = unsafe { std::slice::from_raw_parts_mut(values.items, values.count) };
        for item in items {
            // SAFETY: item buffers originate from this library.
            unsafe { syntax_color_string_free(item) };
        }
        // SAFETY: items/count originate from strings_to_ffi in this library.
        unsafe { free_boxed_slice(values.items, values.count) };
    }
    *values = SyntaxColorStringArray::default();
}

/// Frees a highlight result previously returned by this library.
///
/// # Safety
///
/// - `result` must be null or point to a `SyntaxColorHighlightResult` that is either empty (zeroed)
///   or was filled in by `syntax_color_highlighter_highlight` and not modified since.
/// - The attrs, spans and capture-name strings must not be read after this call. The struct is
///   reset to empty, so freeing it again is a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_highlight_result_free(
    result: *mut SyntaxColorHighlightResult,
) {
    if result.is_null() {
        return;
    }
    // SAFETY: result pointer validated above.
    let result = unsafe { &mut *result };
    if !result.attrs.is_null() {
        // SAFETY: attrs/count originate from highlight_result_to_ffi in this library.
        let attrs = unsafe { std::slice::from_raw_parts_mut(result.attrs, result.attr_count) };
        for attr in attrs {
            // SAFETY: capture_name buffers originate from this library.
            unsafe { syntax_color_string_free(&mut attr.capture_name) };
        }
        // SAFETY: attrs/count originate from this library.
        unsafe { free_boxed_slice(result.attrs, result.attr_count) };
    }
    // SAFETY: spans/count originate from this library.
    unsafe { free_boxed_slice(result.spans, result.span_count) };
    *result = SyntaxColorHighlightResult::default();
}

/// Frees a styled span buffer previously returned by this library.
///
/// # Safety
///
/// - `spans` must be null or point to a `SyntaxColorStyledSpanBuffer` that is either empty (zeroed)
///   or was filled in by `syntax_color_resolve_styled_spans` and not modified since.
/// - The spans must not be read after this call. The struct is reset to empty, so freeing it again
///   is a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_styled_spans_free(spans: *mut SyntaxColorStyledSpanBuffer) {
    if spans.is_null() {
        return;
    }
    // SAFETY: spans pointer validated above.
    let spans = unsafe { &mut *spans };
    // SAFETY: spans/count originate from styled_spans_to_ffi in this library.
    unsafe { free_boxed_slice(spans.spans, spans.count) };
    *spans = SyntaxColorStyledSpanBuffer::default();
}

/// Frees a theme handle previously returned by this library.
///
/// # Safety
///
/// - `theme` must be null or a handle returned by `theme_engine_theme_load_builtin` or
///   `theme_engine_theme_load_json` that has not already been freed.
/// - The handle must not be used after this call, including by calls still running on another
///   thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn theme_engine_theme_free(theme: *mut ThemeEngineTheme) {
    if theme.is_null() {
        return;
    }
    // SAFETY: pointer originated from Box::into_raw in this library.
    let _ = unsafe { Box::from_raw(theme) };
}

/// Loads a built-in theme by name (for example `"tokyonight-dark"`).
///
/// # Safety
///
/// - `name` must be null or point to a NUL-terminated string that stays valid for the duration of
///   the call.
/// - `out_theme` must be null or valid for writing one `ThemeEngineTheme *`. On success it receives
///   a new handle that must be released with `theme_engine_theme_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn theme_engine_theme_load_builtin(
    name: *const c_char,
    out_theme: *mut *mut ThemeEngineTheme,
) -> i32 {
    if out_theme.is_null() {
        return SYNTAX_COLOR_FFI_ERR_NULL;
    }
    // SAFETY: parse_cstr validates null and UTF-8.
    let name = match unsafe { parse_cstr(name) } {
        Ok(name) => name,
        Err(code) => return code,
    };

    let theme = match load_theme(name) {
        Ok(theme) => theme,
        Err(_) => return SYNTAX_COLOR_FFI_ERR_THEME,
    };

    let boxed = Box::new(ThemeEngineTheme { theme });
    // SAFETY: out_theme non-null checked above.
    unsafe { *out_theme = Box::into_raw(boxed) };
    SYNTAX_COLOR_FFI_OK
}

/// Loads a theme from a JSON string.
///
/// # Safety
///
/// - `json` must be null or point to a NUL-terminated string that stays valid for the duration of
///   the call.
/// - `out_theme` must be null or valid for writing one `ThemeEngineTheme *`. On success it receives
///   a new handle that must be released with `theme_engine_theme_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn theme_engine_theme_load_json(
    json: *const c_char,
    out_theme: *mut *mut ThemeEngineTheme,
) -> i32 {
    if out_theme.is_null() {
        return SYNTAX_COLOR_FFI_ERR_NULL;
    }
    // SAFETY: parse_cstr validates null and UTF-8.
    let json = match unsafe { parse_cstr(json) } {
        Ok(json) => json,
        Err(code) => return code,
    };

    let theme = match Theme::from_json_str(json) {
        Ok(theme) => theme,
        Err(_) => return SYNTAX_COLOR_FFI_ERR_THEME,
    };

    let boxed = Box::new(ThemeEngineTheme { theme });
    // SAFETY: out_theme non-null checked above.
    unsafe { *out_theme = Box::into_raw(boxed) };
    SYNTAX_COLOR_FFI_OK
}

/// Resolves a syntax capture style (for example `"@keyword"`).
///
/// # Safety
///
/// - `theme` must be null or a handle returned by `theme_engine_theme_load_builtin` or
///   `theme_engine_theme_load_json` that has not been passed to `theme_engine_theme_free`.
/// - `capture_name` must be null or point to a NUL-terminated string that stays valid for the
///   duration of the call.
/// - `out_style` must be null or valid for writing one `ThemeEngineStyle`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn theme_engine_theme_resolve_capture(
    theme: *const ThemeEngineTheme,
    capture_name: *const c_char,
    out_style: *mut ThemeEngineStyle,
) -> i32 {
    if out_style.is_null() {
        return SYNTAX_COLOR_FFI_ERR_NULL;
    }
    // SAFETY: parse_cstr validates null and UTF-8.
    let capture_name = match unsafe { parse_cstr(capture_name) } {
        Ok(name) => name,
        Err(code) => return code,
    };
    // SAFETY: theme_ref validates theme pointer.
    let theme = match unsafe { theme_ref(theme) } {
        Ok(theme) => theme,
        Err(code) => return code,
    };

    let Some(style) = theme.resolve(capture_name).copied() else {
        return SYNTAX_COLOR_FFI_ERR_NOT_FOUND;
    };
    // SAFETY: out_style checked non-null above.
    unsafe { *out_style = style_to_ffi(style) };
    SYNTAX_COLOR_FFI_OK
}

/// Resolves a UI role style (for example `"statusline"` or `"tab_active"`).
///
/// # Safety
///
/// - `theme` must be null or a handle returned by `theme_engine_theme_load_builtin` or
///   `theme_engine_theme_load_json` that has not been passed to `theme_engine_theme_free`.
/// - `role_name` must be null or point to a NUL-terminated string that stays valid for the duration
///   of the call.
/// - `out_style` must be null or valid for writing one `ThemeEngineStyle`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn theme_engine_theme_resolve_ui(
    theme: *const ThemeEngineTheme,
    role_name: *const c_char,
    out_style: *mut ThemeEngineStyle,
) -> i32 {
    if out_style.is_null() {
        return SYNTAX_COLOR_FFI_ERR_NULL;
    }
    // SAFETY: parse_cstr validates null and UTF-8.
    let role_name = match unsafe { parse_cstr(role_name) } {
        Ok(name) => name,
        Err(code) => return code,
    };
    // SAFETY: theme_ref validates theme pointer.
    let theme = match unsafe { theme_ref(theme) } {
        Ok(theme) => theme,
        Err(code) => return code,
    };

    let Some(style) = theme.resolve_ui(role_name) else {
        return SYNTAX_COLOR_FFI_ERR_NOT_FOUND;
    };
    // SAFETY: out_style checked non-null above.
    unsafe { *out_style = style_to_ffi(style) };
    SYNTAX_COLOR_FFI_OK
}

/// Returns default terminal foreground/background colors from the theme.
///
/// # Safety
///
/// - `theme` must be null or a handle returned by `theme_engine_theme_load_builtin` or
///   `theme_engine_theme_load_json` that has not been passed to `theme_engine_theme_free`.
/// - `out_has_fg` and `out_has_bg` must each be null or valid for writing one `uint8_t`.
/// - `out_fg` and `out_bg` must each be null or valid for writing one `ThemeEngineRgb`. Either may
///   be null when the theme has no such color; if the color exists and its pointer is null, the
///   call returns `SYNTAX_COLOR_FFI_ERR_NULL` after writing the `out_has_*` flags.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn theme_engine_theme_default_terminal_colors(
    theme: *const ThemeEngineTheme,
    out_has_fg: *mut u8,
    out_fg: *mut ThemeEngineRgb,
    out_has_bg: *mut u8,
    out_bg: *mut ThemeEngineRgb,
) -> i32 {
    if out_has_fg.is_null() || out_has_bg.is_null() {
        return SYNTAX_COLOR_FFI_ERR_NULL;
    }
    // SAFETY: theme_ref validates theme pointer.
    let theme = match unsafe { theme_ref(theme) } {
        Ok(theme) => theme,
        Err(code) => return code,
    };
    let (fg, bg) = theme.default_terminal_colors();

    // SAFETY: out_has_* pointers checked non-null above.
    unsafe {
        *out_has_fg = u8::from(fg.is_some());
        *out_has_bg = u8::from(bg.is_some());
    }

    if let Some(color) = fg {
        if out_fg.is_null() {
            return SYNTAX_COLOR_FFI_ERR_NULL;
        }
        // SAFETY: validated non-null when fg exists.
        unsafe { *out_fg = rgb_to_ffi(color) };
    }
    if let Some(color) = bg {
        if out_bg.is_null() {
            return SYNTAX_COLOR_FFI_ERR_NULL;
        }
        // SAFETY: validated non-null when bg exists.
        unsafe { *out_bg = rgb_to_ffi(color) };
    }

    SYNTAX_COLOR_FFI_OK
}

/// Creates a reusable Tree-sitter highlighter handle.
///
/// # Safety
///
/// - `out_highlighter` must be null or valid for writing one `SyntaxColorHighlighter *`. On success
///   it receives a new handle that must be released with `syntax_color_highlighter_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_highlighter_new(
    out_highlighter: *mut *mut SyntaxColorHighlighter,
) -> i32 {
    if out_highlighter.is_null() {
        return SYNTAX_COLOR_FFI_ERR_NULL;
    }

    let highlighter = match SpanHighlighter::new() {
        Ok(highlighter) => highlighter,
        Err(_) => return SYNTAX_COLOR_FFI_ERR_HIGHLIGHT,
    };

    let boxed = Box::new(SyntaxColorHighlighter { highlighter });
    // SAFETY: out_highlighter checked non-null above.
    unsafe { *out_highlighter = Box::into_raw(boxed) };
    SYNTAX_COLOR_FFI_OK
}

/// Frees a highlighter handle previously returned by this library.
///
/// # Safety
///
/// - `highlighter` must be null or a handle returned by `syntax_color_highlighter_new` that has not
///   already been freed.
/// - The handle must not be used after this call, including by calls still running on another
///   thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_highlighter_free(highlighter: *mut SyntaxColorHighlighter) {
    if highlighter.is_null() {
        return;
    }
    // SAFETY: pointer originated from Box::into_raw in this library.
    let _ = unsafe { Box::from_raw(highlighter) };
}

/// Highlights a source buffer into attribute and span arrays.
///
/// # Safety
///
/// - `highlighter` must be null or a handle returned by `syntax_color_highlighter_new` that has not
///   been freed, and must not be in use by another thread during the call.
/// - `source` must be null or point to at least `source_len` readable bytes (null is accepted only
///   when `source_len` is 0).
/// - `out_result` must be null or valid for writing one `SyntaxColorHighlightResult`. Any previous
///   contents are overwritten without being freed; release the result with
///   `syntax_color_highlight_result_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_highlighter_highlight(
    highlighter: *mut SyntaxColorHighlighter,
    source: *const u8,
    source_len: usize,
    grammar: i32,
    out_result: *mut SyntaxColorHighlightResult,
) -> i32 {
    if highlighter.is_null() || out_result.is_null() {
        return SYNTAX_COLOR_FFI_ERR_NULL;
    }
    // SAFETY: parse_bytes validates null/len pair.
    let source = match unsafe { parse_bytes(source, source_len) } {
        Ok(source) => source,
        Err(code) => return code,
    };
    let grammar = match parse_grammar(grammar) {
        Ok(grammar) => grammar,
        Err(code) => return code,
    };
    // SAFETY: highlighter pointer checked non-null above.
    let highlighter = unsafe { &mut *highlighter };

    let result = match highlighter.highlighter.highlight(source, grammar) {
        Ok(result) => result,
        Err(_) => return SYNTAX_COLOR_FFI_ERR_HIGHLIGHT,
    };

    // SAFETY: out_result checked non-null above.
    unsafe { *out_result = highlight_result_to_ffi(result) };
    SYNTAX_COLOR_FFI_OK
}

/// Resolves highlight output into render-ready styled spans.
///
/// # Safety
///
/// - `theme` must be null or a handle returned by `theme_engine_theme_load_builtin` or
///   `theme_engine_theme_load_json` that has not been passed to `theme_engine_theme_free`.
/// - `highlight` must be null or point to a `SyntaxColorHighlightResult` whose `attrs` and `spans`
///   point to at least `attr_count` and `span_count` readable elements (each may be null only when
///   its count is 0), and whose capture-name strings point to at least `len` readable bytes. A
///   result filled in by `syntax_color_highlighter_highlight` and not yet freed satisfies this.
/// - `out_spans` must be null or valid for writing one `SyntaxColorStyledSpanBuffer`. Any previous
///   contents are overwritten without being freed; release the result with
///   `syntax_color_styled_spans_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_resolve_styled_spans(
    theme: *const ThemeEngineTheme,
    source_len: usize,
    highlight: *const SyntaxColorHighlightResult,
    fill_uncovered_with_normal: u8,
    out_spans: *mut SyntaxColorStyledSpanBuffer,
) -> i32 {
    if out_spans.is_null() {
        return SYNTAX_COLOR_FFI_ERR_NULL;
    }
    // SAFETY: theme_ref validates theme pointer.
    let theme = match unsafe { theme_ref(theme) } {
        Ok(theme) => theme,
        Err(code) => return code,
    };
    // SAFETY: highlight_result_from_ffi validates the payload.
    let highlight = match unsafe { highlight_result_from_ffi(highlight) } {
        Ok(highlight) => highlight,
        Err(code) => return code,
    };

    let styled = if fill_uncovered_with_normal != 0 {
        match resolve_styled_spans_for_source(source_len, &highlight, theme) {
            Ok(styled) => styled,
            Err(_) => return SYNTAX_COLOR_FFI_ERR_RENDER,
        }
    } else {
        match resolve_styled_spans(&highlight, theme) {
            Ok(styled) => styled,
            Err(_) => return SYNTAX_COLOR_FFI_ERR_RENDER,
        }
    };

    // SAFETY: out_spans checked non-null above.
    unsafe { *out_spans = styled_spans_to_ffi(styled) };
    SYNTAX_COLOR_FFI_OK
}

/// Renders styled spans into a single ANSI/VT byte buffer.
///
/// # Safety
///
/// - `source` must be null or point to at least `source_len` readable bytes (null is accepted only
///   when `source_len` is 0).
/// - `spans` must be null or point to at least `span_count` readable `SyntaxColorStyledSpan` values
///   (null is accepted only when `span_count` is 0).
/// - `out_ansi` must be null or valid for writing one `SyntaxColorString`. Any previous contents
///   are overwritten without being freed; release the result with `syntax_color_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_render_ansi(
    source: *const u8,
    source_len: usize,
    spans: *const SyntaxColorStyledSpan,
    span_count: usize,
    color_mode: i32,
    preserve_terminal_background: u8,
    out_ansi: *mut SyntaxColorString,
) -> i32 {
    // SAFETY: parse_bytes validates null/len pair.
    let source = match unsafe { parse_bytes(source, source_len) } {
        Ok(source) => source,
        Err(code) => return code,
    };
    // SAFETY: styled_spans_from_ffi validates the payload.
    let spans = match unsafe { styled_spans_from_ffi(spans, span_count) } {
        Ok(spans) => spans,
        Err(code) => return code,
    };
    let color_mode = match parse_color_mode(color_mode) {
        Ok(color_mode) => color_mode,
        Err(code) => return code,
    };

    let ansi = match render_ansi(
        source,
        &spans,
        color_mode,
        preserve_terminal_background != 0,
    ) {
        Ok(ansi) => ansi,
        Err(_) => return SYNTAX_COLOR_FFI_ERR_RENDER,
    };

    write_string_output(out_ansi, ansi)
}

/// Renders styled spans into line-split ANSI/VT byte buffers.
///
/// # Safety
///
/// - `source` must be null or point to at least `source_len` readable bytes (null is accepted only
///   when `source_len` is 0).
/// - `spans` must be null or point to at least `span_count` readable `SyntaxColorStyledSpan` values
///   (null is accepted only when `span_count` is 0).
/// - `out_lines` must be null or valid for writing one `SyntaxColorStringArray`. Any previous
///   contents are overwritten without being freed; release the result with
///   `syntax_color_string_array_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_render_ansi_lines(
    source: *const u8,
    source_len: usize,
    spans: *const SyntaxColorStyledSpan,
    span_count: usize,
    color_mode: i32,
    preserve_terminal_background: u8,
    out_lines: *mut SyntaxColorStringArray,
) -> i32 {
    // SAFETY: parse_bytes validates null/len pair.
    let source = match unsafe { parse_bytes(source, source_len) } {
        Ok(source) => source,
        Err(code) => return code,
    };
    // SAFETY: styled_spans_from_ffi validates the payload.
    let spans = match unsafe { styled_spans_from_ffi(spans, span_count) } {
        Ok(spans) => spans,
        Err(code) => return code,
    };
    let color_mode = match parse_color_mode(color_mode) {
        Ok(color_mode) => color_mode,
        Err(code) => return code,
    };

    let lines = match render_ansi_lines(
        source,
        &spans,
        color_mode,
        preserve_terminal_background != 0,
    ) {
        Ok(lines) => lines,
        Err(_) => return SYNTAX_COLOR_FFI_ERR_RENDER,
    };

    write_string_array_output(out_lines, lines)
}

/// Highlights and renders a source buffer into ANSI/VT in one call.
///
/// Pass `NULL` for `highlighter` to use a temporary one-shot highlighter.
///
/// # Safety
///
/// - `highlighter` must be null (a temporary highlighter is used) or a handle returned by
///   `syntax_color_highlighter_new` that has not been freed, and must not be in use by another
///   thread during the call.
/// - `theme` must be null or a handle returned by `theme_engine_theme_load_builtin` or
///   `theme_engine_theme_load_json` that has not been passed to `theme_engine_theme_free`.
/// - `source` must be null or point to at least `source_len` readable bytes (null is accepted only
///   when `source_len` is 0).
/// - `out_ansi` must be null or valid for writing one `SyntaxColorString`. Any previous contents
///   are overwritten without being freed; release the result with `syntax_color_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_highlight_to_ansi(
    highlighter: *mut SyntaxColorHighlighter,
    theme: *const ThemeEngineTheme,
    source: *const u8,
    source_len: usize,
    grammar: i32,
    color_mode: i32,
    preserve_terminal_background: u8,
    out_ansi: *mut SyntaxColorString,
) -> i32 {
    // SAFETY: parse_bytes validates null/len pair.
    let source = match unsafe { parse_bytes(source, source_len) } {
        Ok(source) => source,
        Err(code) => return code,
    };
    // SAFETY: theme_ref validates theme pointer.
    let theme = match unsafe { theme_ref(theme) } {
        Ok(theme) => theme,
        Err(code) => return code,
    };
    let grammar = match parse_grammar(grammar) {
        Ok(grammar) => grammar,
        Err(code) => return code,
    };
    let color_mode = match parse_color_mode(color_mode) {
        Ok(color_mode) => color_mode,
        Err(code) => return code,
    };

    // SAFETY: with_optional_highlighter validates nullable handle semantics.
    let ansi = match unsafe {
        with_optional_highlighter(highlighter, |highlighter| {
            render_highlight_to_ansi(
                source,
                grammar,
                theme,
                Some(highlighter),
                Some(color_mode),
                Some(preserve_terminal_background != 0),
            )
            .map_err(|_| SYNTAX_COLOR_FFI_ERR_RENDER)
        })
    } {
        Ok(ansi) => ansi,
        Err(code) => return code,
    };

    write_string_output(out_ansi, ansi)
}

/// Highlights and renders a source buffer into line-split ANSI/VT output.
///
/// Pass `NULL` for `highlighter` to use a temporary one-shot highlighter.
///
/// # Safety
///
/// - `highlighter` must be null (a temporary highlighter is used) or a handle returned by
///   `syntax_color_highlighter_new` that has not been freed, and must not be in use by another
///   thread during the call.
/// - `theme` must be null or a handle returned by `theme_engine_theme_load_builtin` or
///   `theme_engine_theme_load_json` that has not been passed to `theme_engine_theme_free`.
/// - `source` must be null or point to at least `source_len` readable bytes (null is accepted only
///   when `source_len` is 0).
/// - `out_lines` must be null or valid for writing one `SyntaxColorStringArray`. Any previous
///   contents are overwritten without being freed; release the result with
///   `syntax_color_string_array_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_highlight_to_ansi_lines(
    highlighter: *mut SyntaxColorHighlighter,
    theme: *const ThemeEngineTheme,
    source: *const u8,
    source_len: usize,
    grammar: i32,
    color_mode: i32,
    preserve_terminal_background: u8,
    out_lines: *mut SyntaxColorStringArray,
) -> i32 {
    // SAFETY: parse_bytes validates null/len pair.
    let source = match unsafe { parse_bytes(source, source_len) } {
        Ok(source) => source,
        Err(code) => return code,
    };
    // SAFETY: theme_ref validates theme pointer.
    let theme = match unsafe { theme_ref(theme) } {
        Ok(theme) => theme,
        Err(code) => return code,
    };
    let grammar = match parse_grammar(grammar) {
        Ok(grammar) => grammar,
        Err(code) => return code,
    };
    let color_mode = match parse_color_mode(color_mode) {
        Ok(color_mode) => color_mode,
        Err(code) => return code,
    };

    // SAFETY: with_optional_highlighter validates nullable handle semantics.
    let lines = match unsafe {
        with_optional_highlighter(highlighter, |highlighter| {
            let highlight = highlighter
                .highlight(source, grammar)
                .map_err(|_| SYNTAX_COLOR_FFI_ERR_HIGHLIGHT)?;
            let styled = resolve_styled_spans_for_source(source.len(), &highlight, theme)
                .map_err(|_| SYNTAX_COLOR_FFI_ERR_RENDER)?;
            render_ansi_lines(
                source,
                &styled,
                color_mode,
                preserve_terminal_background != 0,
            )
            .map_err(|_| SYNTAX_COLOR_FFI_ERR_RENDER)
        })
    } {
        Ok(lines) => lines,
        Err(code) => return code,
    };

    write_string_array_output(out_lines, lines)
}

/// Builds OSC 10/11 sequences from the theme's default terminal colors.
///
/// # Safety
///
/// - `theme` must be null or a handle returned by `theme_engine_theme_load_builtin` or
///   `theme_engine_theme_load_json` that has not been passed to `theme_engine_theme_free`.
/// - `out_osc` must be null or valid for writing one `SyntaxColorString`. Any previous contents are
///   overwritten without being freed; release the result with `syntax_color_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_osc_set_default_colors(
    theme: *const ThemeEngineTheme,
    out_osc: *mut SyntaxColorString,
) -> i32 {
    // SAFETY: theme_ref validates theme pointer.
    let theme = match unsafe { theme_ref(theme) } {
        Ok(theme) => theme,
        Err(code) => return code,
    };
    write_string_output(out_osc, osc_set_default_colors(theme.clone()))
}

/// Returns OSC 110 to reset terminal default foreground color.
///
/// # Safety
///
/// - `out_osc` must be null or valid for writing one `SyntaxColorString`. Any previous contents are
///   overwritten without being freed; release the result with `syntax_color_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_osc_reset_default_foreground(
    out_osc: *mut SyntaxColorString,
) -> i32 {
    write_string_output(out_osc, osc_reset_default_foreground().to_owned())
}

/// Returns OSC 111 to reset terminal default background color.
///
/// # Safety
///
/// - `out_osc` must be null or valid for writing one `SyntaxColorString`. Any previous contents are
///   overwritten without being freed; release the result with `syntax_color_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_osc_reset_default_background(
    out_osc: *mut SyntaxColorString,
) -> i32 {
    write_string_output(out_osc, osc_reset_default_background().to_owned())
}

/// Returns OSC 110 + 111 to reset terminal default foreground/background colors.
///
/// # Safety
///
/// - `out_osc` must be null or valid for writing one `SyntaxColorString`. Any previous contents are
///   overwritten without being freed; release the result with `syntax_color_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_osc_reset_default_colors(
    out_osc: *mut SyntaxColorString,
) -> i32 {
    write_string_output(out_osc, osc_reset_default_colors().to_owned())
}

/// Creates a multi-line incremental VT patch renderer.
///
/// # Safety
///
/// - `out_renderer` must be null or valid for writing one `SyntaxColorIncrementalRenderer *`. On
///   success it receives a new handle that must be released with
///   `syntax_color_incremental_renderer_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_incremental_renderer_new(
    width: usize,
    height: usize,
    out_renderer: *mut *mut SyntaxColorIncrementalRenderer,
) -> i32 {
    if out_renderer.is_null() {
        return SYNTAX_COLOR_FFI_ERR_NULL;
    }
    let boxed = Box::new(SyntaxColorIncrementalRenderer {
        renderer: render_ansi::ansi_structures::IncrementalRenderer::new(width, height),
    });
    // SAFETY: out_renderer checked non-null above.
    unsafe { *out_renderer = Box::into_raw(boxed) };
    SYNTAX_COLOR_FFI_OK
}

/// Frees an incremental renderer handle previously returned by this library.
///
/// # Safety
///
/// - `renderer` must be null or a handle returned by `syntax_color_incremental_renderer_new` that
///   has not already been freed.
/// - The handle must not be used after this call, including by calls still running on another
///   thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_incremental_renderer_free(
    renderer: *mut SyntaxColorIncrementalRenderer,
) {
    if renderer.is_null() {
        return;
    }
    // SAFETY: pointer originated from Box::into_raw in this library.
    let _ = unsafe { Box::from_raw(renderer) };
}

/// Resizes the incremental renderer viewport.
///
/// # Safety
///
/// - `renderer` must be null or a handle returned by `syntax_color_incremental_renderer_new` that
///   has not been freed, and must not be in use by another thread during the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_incremental_renderer_resize(
    renderer: *mut SyntaxColorIncrementalRenderer,
    width: usize,
    height: usize,
) -> i32 {
    // SAFETY: renderer_mut validates pointer.
    let renderer = match unsafe { renderer_mut(renderer) } {
        Ok(renderer) => renderer,
        Err(code) => return code,
    };
    renderer.resize(width, height);
    SYNTAX_COLOR_FFI_OK
}

/// Clears cached frame state from the incremental renderer.
///
/// # Safety
///
/// - `renderer` must be null or a handle returned by `syntax_color_incremental_renderer_new` that
///   has not been freed, and must not be in use by another thread during the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_incremental_renderer_clear_state(
    renderer: *mut SyntaxColorIncrementalRenderer,
) -> i32 {
    // SAFETY: renderer_mut validates pointer.
    let renderer = match unsafe { renderer_mut(renderer) } {
        Ok(renderer) => renderer,
        Err(code) => return code,
    };
    renderer.clear_state();
    SYNTAX_COLOR_FFI_OK
}

/// Sets the terminal origin used by the incremental renderer.
///
/// # Safety
///
/// - `renderer` must be null or a handle returned by `syntax_color_incremental_renderer_new` that
///   has not been freed, and must not be in use by another thread during the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_incremental_renderer_set_origin(
    renderer: *mut SyntaxColorIncrementalRenderer,
    row: usize,
    col: usize,
) -> i32 {
    // SAFETY: renderer_mut validates pointer.
    let renderer = match unsafe { renderer_mut(renderer) } {
        Ok(renderer) => renderer,
        Err(code) => return code,
    };
    renderer.set_origin(row, col);
    SYNTAX_COLOR_FFI_OK
}

/// Sets the color mode used by the incremental renderer.
///
/// # Safety
///
/// - `renderer` must be null or a handle returned by `syntax_color_incremental_renderer_new` that
///   has not been freed, and must not be in use by another thread during the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_incremental_renderer_set_color_mode(
    renderer: *mut SyntaxColorIncrementalRenderer,
    color_mode: i32,
) -> i32 {
    // SAFETY: renderer_mut validates pointer.
    let renderer = match unsafe { renderer_mut(renderer) } {
        Ok(renderer) => renderer,
        Err(code) => return code,
    };
    let color_mode = match parse_color_mode(color_mode) {
        Ok(color_mode) => color_mode,
        Err(code) => return code,
    };
    renderer.set_color_mode(color_mode);
    SYNTAX_COLOR_FFI_OK
}

/// Sets whether the incremental renderer preserves terminal background color.
///
/// # Safety
///
/// - `renderer` must be null or a handle returned by `syntax_color_incremental_renderer_new` that
///   has not been freed, and must not be in use by another thread during the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_incremental_renderer_set_preserve_terminal_background(
    renderer: *mut SyntaxColorIncrementalRenderer,
    preserve_terminal_background: u8,
) -> i32 {
    // SAFETY: renderer_mut validates pointer.
    let renderer = match unsafe { renderer_mut(renderer) } {
        Ok(renderer) => renderer,
        Err(code) => return code,
    };
    renderer.set_preserve_terminal_background(preserve_terminal_background != 0);
    SYNTAX_COLOR_FFI_OK
}

/// Renders a VT patch from styled spans using the incremental renderer.
///
/// # Safety
///
/// - `renderer` must be null or a handle returned by `syntax_color_incremental_renderer_new` that
///   has not been freed, and must not be in use by another thread during the call.
/// - `source` must be null or point to at least `source_len` readable bytes (null is accepted only
///   when `source_len` is 0).
/// - `spans` must be null or point to at least `span_count` readable `SyntaxColorStyledSpan` values
///   (null is accepted only when `span_count` is 0).
/// - `out_patch` must be null or valid for writing one `SyntaxColorString`. Any previous contents
///   are overwritten without being freed; release the result with `syntax_color_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_incremental_renderer_render_patch(
    renderer: *mut SyntaxColorIncrementalRenderer,
    source: *const u8,
    source_len: usize,
    spans: *const SyntaxColorStyledSpan,
    span_count: usize,
    out_patch: *mut SyntaxColorString,
) -> i32 {
    // SAFETY: renderer_mut validates pointer.
    let renderer = match unsafe { renderer_mut(renderer) } {
        Ok(renderer) => renderer,
        Err(code) => return code,
    };
    // SAFETY: parse_bytes validates null/len pair.
    let source = match unsafe { parse_bytes(source, source_len) } {
        Ok(source) => source,
        Err(code) => return code,
    };
    // SAFETY: styled_spans_from_ffi validates the payload.
    let spans = match unsafe { styled_spans_from_ffi(spans, span_count) } {
        Ok(spans) => spans,
        Err(code) => return code,
    };

    let patch = match renderer.render_patch(source, &spans) {
        Ok(patch) => patch,
        Err(_) => return SYNTAX_COLOR_FFI_ERR_RENDER,
    };

    write_string_output(out_patch, patch)
}

/// Highlights and renders a VT patch using the incremental renderer.
///
/// Pass `NULL` for `highlighter` to use a temporary one-shot highlighter.
///
/// # Safety
///
/// - `renderer` must be null or a handle returned by `syntax_color_incremental_renderer_new` that
///   has not been freed, and must not be in use by another thread during the call.
/// - `highlighter` must be null (a temporary highlighter is used) or a handle returned by
///   `syntax_color_highlighter_new` that has not been freed, and must not be in use by another
///   thread during the call.
/// - `theme` must be null or a handle returned by `theme_engine_theme_load_builtin` or
///   `theme_engine_theme_load_json` that has not been passed to `theme_engine_theme_free`.
/// - `source` must be null or point to at least `source_len` readable bytes (null is accepted only
///   when `source_len` is 0).
/// - `out_patch` must be null or valid for writing one `SyntaxColorString`. Any previous contents
///   are overwritten without being freed; release the result with `syntax_color_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_incremental_renderer_highlight_to_patch(
    renderer: *mut SyntaxColorIncrementalRenderer,
    highlighter: *mut SyntaxColorHighlighter,
    theme: *const ThemeEngineTheme,
    source: *const u8,
    source_len: usize,
    grammar: i32,
    out_patch: *mut SyntaxColorString,
) -> i32 {
    // SAFETY: renderer_mut validates pointer.
    let renderer = match unsafe { renderer_mut(renderer) } {
        Ok(renderer) => renderer,
        Err(code) => return code,
    };
    // SAFETY: parse_bytes validates null/len pair.
    let source = match unsafe { parse_bytes(source, source_len) } {
        Ok(source) => source,
        Err(code) => return code,
    };
    // SAFETY: theme_ref validates theme pointer.
    let theme = match unsafe { theme_ref(theme) } {
        Ok(theme) => theme,
        Err(code) => return code,
    };
    let grammar = match parse_grammar(grammar) {
        Ok(grammar) => grammar,
        Err(code) => return code,
    };

    // SAFETY: with_optional_highlighter validates nullable handle semantics.
    let patch = match unsafe {
        with_optional_highlighter(highlighter, |highlighter| {
            renderer
                .highlight_to_patch(highlighter, source, grammar, theme)
                .map_err(|_| SYNTAX_COLOR_FFI_ERR_RENDER)
        })
    } {
        Ok(patch) => patch,
        Err(code) => return code,
    };

    write_string_output(out_patch, patch)
}

/// Creates a single-line stream-safe VT patch renderer.
///
/// # Safety
///
/// - `out_renderer` must be null or valid for writing one `SyntaxColorStreamLineRenderer *`. On
///   success it receives a new handle that must be released with
///   `syntax_color_stream_line_renderer_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_stream_line_renderer_new(
    out_renderer: *mut *mut SyntaxColorStreamLineRenderer,
) -> i32 {
    if out_renderer.is_null() {
        return SYNTAX_COLOR_FFI_ERR_NULL;
    }
    let boxed = Box::new(SyntaxColorStreamLineRenderer {
        renderer: render_ansi::ansi_structures::StreamLineRenderer::new(),
    });
    // SAFETY: out_renderer checked non-null above.
    unsafe { *out_renderer = Box::into_raw(boxed) };
    SYNTAX_COLOR_FFI_OK
}

/// Frees a stream-line renderer handle previously returned by this library.
///
/// # Safety
///
/// - `renderer` must be null or a handle returned by `syntax_color_stream_line_renderer_new` that
///   has not already been freed.
/// - The handle must not be used after this call, including by calls still running on another
///   thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_stream_line_renderer_free(
    renderer: *mut SyntaxColorStreamLineRenderer,
) {
    if renderer.is_null() {
        return;
    }
    // SAFETY: pointer originated from Box::into_raw in this library.
    let _ = unsafe { Box::from_raw(renderer) };
}

/// Clears cached line state from the stream-line renderer.
///
/// # Safety
///
/// - `renderer` must be null or a handle returned by `syntax_color_stream_line_renderer_new` that
///   has not been freed, and must not be in use by another thread during the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_stream_line_renderer_clear_state(
    renderer: *mut SyntaxColorStreamLineRenderer,
) -> i32 {
    // SAFETY: stream_renderer_mut validates pointer.
    let renderer = match unsafe { stream_renderer_mut(renderer) } {
        Ok(renderer) => renderer,
        Err(code) => return code,
    };
    renderer.clear_state();
    SYNTAX_COLOR_FFI_OK
}

/// Sets the color mode used by the stream-line renderer.
///
/// # Safety
///
/// - `renderer` must be null or a handle returned by `syntax_color_stream_line_renderer_new` that
///   has not been freed, and must not be in use by another thread during the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_stream_line_renderer_set_color_mode(
    renderer: *mut SyntaxColorStreamLineRenderer,
    color_mode: i32,
) -> i32 {
    // SAFETY: stream_renderer_mut validates pointer.
    let renderer = match unsafe { stream_renderer_mut(renderer) } {
        Ok(renderer) => renderer,
        Err(code) => return code,
    };
    let color_mode = match parse_color_mode(color_mode) {
        Ok(color_mode) => color_mode,
        Err(code) => return code,
    };
    renderer.set_color_mode(color_mode);
    SYNTAX_COLOR_FFI_OK
}

/// Sets whether the stream-line renderer preserves terminal background color.
///
/// # Safety
///
/// - `renderer` must be null or a handle returned by `syntax_color_stream_line_renderer_new` that
///   has not been freed, and must not be in use by another thread during the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_stream_line_renderer_set_preserve_terminal_background(
    renderer: *mut SyntaxColorStreamLineRenderer,
    preserve_terminal_background: u8,
) -> i32 {
    // SAFETY: stream_renderer_mut validates pointer.
    let renderer = match unsafe { stream_renderer_mut(renderer) } {
        Ok(renderer) => renderer,
        Err(code) => return code,
    };
    renderer.set_preserve_terminal_background(preserve_terminal_background != 0);
    SYNTAX_COLOR_FFI_OK
}

/// Renders a single-line VT patch from styled spans using the stream-line renderer.
///
/// # Safety
///
/// - `renderer` must be null or a handle returned by `syntax_color_stream_line_renderer_new` that
///   has not been freed, and must not be in use by another thread during the call.
/// - `source` must be null or point to at least `source_len` readable bytes (null is accepted only
///   when `source_len` is 0).
/// - `spans` must be null or point to at least `span_count` readable `SyntaxColorStyledSpan` values
///   (null is accepted only when `span_count` is 0).
/// - `out_patch` must be null or valid for writing one `SyntaxColorString`. Any previous contents
///   are overwritten without being freed; release the result with `syntax_color_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_stream_line_renderer_render_line_patch(
    renderer: *mut SyntaxColorStreamLineRenderer,
    source: *const u8,
    source_len: usize,
    spans: *const SyntaxColorStyledSpan,
    span_count: usize,
    out_patch: *mut SyntaxColorString,
) -> i32 {
    // SAFETY: stream_renderer_mut validates pointer.
    let renderer = match unsafe { stream_renderer_mut(renderer) } {
        Ok(renderer) => renderer,
        Err(code) => return code,
    };
    // SAFETY: parse_bytes validates null/len pair.
    let source = match unsafe { parse_bytes(source, source_len) } {
        Ok(source) => source,
        Err(code) => return code,
    };
    // SAFETY: styled_spans_from_ffi validates the payload.
    let spans = match unsafe { styled_spans_from_ffi(spans, span_count) } {
        Ok(spans) => spans,
        Err(code) => return code,
    };

    let patch = match renderer.render_line_patch(source, &spans) {
        Ok(patch) => patch,
        Err(_) => return SYNTAX_COLOR_FFI_ERR_RENDER,
    };

    write_string_output(out_patch, patch)
}

/// Highlights and renders a single-line VT patch using the stream-line renderer.
///
/// Pass `NULL` for `highlighter` to use a temporary one-shot highlighter.
///
/// # Safety
///
/// - `renderer` must be null or a handle returned by `syntax_color_stream_line_renderer_new` that
///   has not been freed, and must not be in use by another thread during the call.
/// - `highlighter` must be null (a temporary highlighter is used) or a handle returned by
///   `syntax_color_highlighter_new` that has not been freed, and must not be in use by another
///   thread during the call.
/// - `theme` must be null or a handle returned by `theme_engine_theme_load_builtin` or
///   `theme_engine_theme_load_json` that has not been passed to `theme_engine_theme_free`.
/// - `source` must be null or point to at least `source_len` readable bytes (null is accepted only
///   when `source_len` is 0).
/// - `out_patch` must be null or valid for writing one `SyntaxColorString`. Any previous contents
///   are overwritten without being freed; release the result with `syntax_color_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn syntax_color_stream_line_renderer_highlight_line_to_patch(
    renderer: *mut SyntaxColorStreamLineRenderer,
    highlighter: *mut SyntaxColorHighlighter,
    theme: *const ThemeEngineTheme,
    source: *const u8,
    source_len: usize,
    grammar: i32,
    out_patch: *mut SyntaxColorString,
) -> i32 {
    // SAFETY: stream_renderer_mut validates pointer.
    let renderer = match unsafe { stream_renderer_mut(renderer) } {
        Ok(renderer) => renderer,
        Err(code) => return code,
    };
    // SAFETY: parse_bytes validates null/len pair.
    let source = match unsafe { parse_bytes(source, source_len) } {
        Ok(source) => source,
        Err(code) => return code,
    };
    // SAFETY: theme_ref validates theme pointer.
    let theme = match unsafe { theme_ref(theme) } {
        Ok(theme) => theme,
        Err(code) => return code,
    };
    let grammar = match parse_grammar(grammar) {
        Ok(grammar) => grammar,
        Err(code) => return code,
    };

    // SAFETY: with_optional_highlighter validates nullable handle semantics.
    let patch = match unsafe {
        with_optional_highlighter(highlighter, |highlighter| {
            renderer
                .highlight_line_to_patch(highlighter, source, grammar, theme)
                .map_err(|_| SYNTAX_COLOR_FFI_ERR_RENDER)
        })
    } {
        Ok(patch) => patch,
        Err(code) => return code,
    };

    write_string_output(out_patch, patch)
}
