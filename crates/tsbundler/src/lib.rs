#![allow(clippy::missing_safety_doc)]

use highlight_spans::highlight_structures::{Grammar, HighlightResult, SpanHighlighter};
use libc::{c_char, c_int, c_uchar, c_void, ssize_t};
use render_ansi::{
    ansi_structures::{ColorMode, StyledSpan},
    common::{render_ansi, resolve_styled_spans_for_source},
};
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    ptr, slice,
    sync::Mutex,
};
use theme_engine::{
    common::{load_theme, normalize_capture_name},
    theme_structures::{available_themes, BuiltinTheme, Rgb as ThemeRgb, Style as ThemeStyle},
};

// =============================================================================
// C type aliases matching zedit.h
// =============================================================================

type ZeditAttr = u16; // zedit_attr_t
type ZeditRgb = u32; // zedit_rgb_t  0x00RRGGBB
type ZeditVduAttr = u32; // zedit_vdu_attr

const ZEDIT_RGBNONE: ZeditRgb = 0xffff_ffff;
const ZEDIT_ATTR_NONE: ZeditAttr = 0;
const ZEDIT_VDU_ATTR_BOLD: ZeditVduAttr = 1 << 0;
const ZEDIT_VDU_ATTR_ITALIC: ZeditVduAttr = 1 << 1;
const ZEDIT_VDU_ATTR_UNDERLINE: ZeditVduAttr = 1 << 2;

// =============================================================================
// Error codes (§3)
// =============================================================================

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsbundlerError {
    Ok = 0,
    ErrNoMem = 1,
    ErrNoLang = 2,
    ErrNoTheme = 3,
    ErrParse = 4,
    ErrInternal = 99,
}

// =============================================================================
// Callback type aliases (§5, §6, §7, §8)
// =============================================================================

type TsbundlerLogFn = Option<unsafe extern "C" fn(*mut c_void, i32, *const c_char)>;
type TsbundlerLangFn =
    Option<unsafe extern "C" fn(*mut c_void, *const c_char, *const *const c_char, c_int)>;
type TsbundlerThemeFn = Option<unsafe extern "C" fn(*mut c_void, *const c_char)>;
type TsbundlerElemFn = Option<
    unsafe extern "C" fn(*mut c_void, *const c_char, ZeditRgb, ZeditRgb, ZeditVduAttr, c_int),
>;

/// TSInputRead-compatible callback (§7.1).
/// Returns a pointer to raw bytes at (byte_offset, row, col) and sets *bytes_read.
type TsbundlerReadFn =
    Option<unsafe extern "C" fn(*mut c_void, u32, u32, u32, *mut u32) -> *const c_char>;

/// Pull callback: zedit returns a zedit_attr_t[nchars] buffer for line `row` (§7.2).
type TsbundlerAttrsFn = Option<unsafe extern "C" fn(*mut c_void, u32, u32) -> *mut ZeditAttr>;

/// Changed-range callback for incremental edit (§7.4).
type TsbundlerRangeFn = Option<unsafe extern "C" fn(*mut c_void, u32, u32)>;

// =============================================================================
// Edit descriptor (§7.4)
// =============================================================================

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TsbundlerEdit {
    pub start_byte: u32,
    pub old_end_byte: u32,
    pub new_end_byte: u32,
    pub start_row: u32,
    pub start_col: u32,
    pub old_end_row: u32,
    pub old_end_col: u32,
    pub new_end_row: u32,
    pub new_end_col: u32,
}

// =============================================================================
// Encoding enum (§7.1)
// =============================================================================

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TsbundlerEncoding {
    Utf8 = 0,
    Utf16Le = 1,
    Utf16Be = 2,
}

// =============================================================================
// Internal helpers
// =============================================================================

struct ThemeElement {
    name: CString,
    fg: ZeditRgb,
    bg: ZeditRgb,
    attrs: ZeditVduAttr,
    index: u16, // zedit_attr_t value (1-based, dense)
}

struct ActivatedTheme {
    c_name: CString,
    elements: Vec<ThemeElement>,
    index_by_style_key: HashMap<String, u16>,
    engine_theme: theme_engine::theme_structures::Theme,
}

fn is_reserved_ui_style_key(name: &str) -> bool {
    matches!(
        name,
        "cursorline"
            | "default_bg"
            | "default_fg"
            | "normal"
            | "selection"
            | "statusline"
            | "statusline_inactive"
            | "tab_active"
            | "tab_inactive"
    )
}

fn rgb_to_packed(rgb: ThemeRgb) -> ZeditRgb {
    (u32::from(rgb.r) << 16) | (u32::from(rgb.g) << 8) | u32::from(rgb.b)
}

fn style_to_vdu_attr(style: &ThemeStyle) -> ZeditVduAttr {
    let mut a = 0u32;
    if style.bold {
        a |= ZEDIT_VDU_ATTR_BOLD;
    }
    if style.italic {
        a |= ZEDIT_VDU_ATTR_ITALIC;
    }
    if style.underline {
        a |= ZEDIT_VDU_ATTR_UNDERLINE;
    }
    a
}

fn canonical_theme_name(name: &str) -> Option<&'static str> {
    match BuiltinTheme::from_name(name)? {
        BuiltinTheme::TokyoNightDark => Some("tokyonight-dark"),
        BuiltinTheme::TokyoNightNight => Some("tokyonight-night"),
        BuiltinTheme::TokyoNightStorm => Some("tokyonight-storm"),
        BuiltinTheme::TokyoNightMoon => Some("tokyonight-moon"),
        BuiltinTheme::TokyoNightLight => Some("tokyonight-light"),
        BuiltinTheme::TokyoNightDay => Some("tokyonight-day"),
        BuiltinTheme::CatppuccinLatte => Some("catppuccin-latte"),
        BuiltinTheme::CatppuccinFrappe => Some("catppuccin-frappe"),
        BuiltinTheme::CatppuccinMacchiato => Some("catppuccin-macchiato"),
        BuiltinTheme::CatppuccinMocha => Some("catppuccin-mocha"),
        BuiltinTheme::Aviel => Some("aviel"),
        BuiltinTheme::StudioDefault => Some("studio-default"),
        BuiltinTheme::SolarizedDark => Some("solarized-dark"),
        BuiltinTheme::SolarizedLight => Some("solarized-light"),
    }
}

impl ActivatedTheme {
    fn from_name(theme_name: &str) -> Result<Self, String> {
        let engine_theme = load_theme(theme_name)
            .map_err(|e| format!("failed to load theme {theme_name}: {e}"))?;

        let style_keys: Vec<String> = engine_theme
            .styles
            .keys()
            .filter(|k| !is_reserved_ui_style_key(k))
            .cloned()
            .collect();

        let mut elements = Vec::with_capacity(style_keys.len());
        let mut index_by_style_key = HashMap::with_capacity(style_keys.len());

        for (i, style_key) in style_keys.iter().enumerate() {
            let style = engine_theme.styles.get(style_key).unwrap();
            let attr_index = u16::try_from(i + 1)
                .map_err(|_| "theme has too many elements for zedit_attr_t".to_string())?;
            elements.push(ThemeElement {
                name: CString::new(format!("@{style_key}")).expect("valid name"),
                fg: style.fg.map(rgb_to_packed).unwrap_or(ZEDIT_RGBNONE),
                bg: style.bg.map(rgb_to_packed).unwrap_or(ZEDIT_RGBNONE),
                attrs: style_to_vdu_attr(style),
                index: attr_index,
            });
            index_by_style_key.insert(style_key.clone(), attr_index);
        }

        Ok(Self {
            c_name: CString::new(theme_name).expect("valid theme name"),
            elements,
            index_by_style_key,
            engine_theme,
        })
    }

    /// Resolve a tree-sitter capture name to a zedit_attr_t index.
    fn resolve_capture_index(&self, capture_name: &str) -> ZeditAttr {
        let mut key = normalize_capture_name(capture_name);
        loop {
            if self.engine_theme.styles.contains_key(&key) {
                if key == "normal" {
                    return ZEDIT_ATTR_NONE;
                }
                return self
                    .index_by_style_key
                    .get(&key)
                    .copied()
                    .unwrap_or(ZEDIT_ATTR_NONE);
            }
            match key.rfind('.') {
                Some(i) => key.truncate(i),
                None => break,
            }
        }
        ZEDIT_ATTR_NONE
    }
}

// =============================================================================
// Language registry — built from Grammar enum; no WASM
// =============================================================================

struct LangEntry {
    canonical: &'static str,
    #[allow(dead_code)]
    extensions: Vec<&'static str>,
    c_name: CString,
    c_extensions: Vec<CString>,
}

fn build_lang_registry() -> Vec<LangEntry> {
    // Map Grammar variants to file extensions.  These mirror the LANGUAGE_ASSETS
    // list that was previously driven by WASM files.
    let grammar_exts: &[(&str, &[&str])] = &[
        ("objectscript_routine", &[".mac", ".inc", ".int", ".rtn"]),
        ("objectscript_udl", &[".cls"]),
        ("objectscript", &[]),
        ("sql", &[".sql"]),
        ("python", &[".py"]),
        ("markdown", &[".md"]),
        ("xml", &[".xml"]),
        ("json", &[".json"]),
        ("yaml", &[".yaml", ".yml"]),
        ("css", &[]),
        ("html", &[".html"]),
        ("javascript", &[".js", ".jsx", ".mjs", ".cjs"]),
        ("jsdoc", &[]),
        ("regex", &[]),
        ("toml", &[".toml"]),
        ("mdx", &[]),
    ];

    grammar_exts
        .iter()
        .map(|&(name, exts)| LangEntry {
            canonical: name,
            extensions: exts.to_vec(),
            c_name: CString::new(name).expect("valid lang name"),
            c_extensions: exts
                .iter()
                .map(|&e| CString::new(e).expect("valid extension"))
                .collect(),
        })
        .collect()
}

// =============================================================================
// tsbundler_ctx  (§4)
// =============================================================================

#[allow(non_camel_case_types)]
pub struct tsbundler_ctx_s {
    highlighter: SpanHighlighter,
    langs: Vec<LangEntry>,
    #[allow(dead_code)]
    lang_lookup: HashMap<String, usize>,
    themes: Vec<ActivatedTheme>,
    theme_lookup: HashMap<String, usize>,
    active_theme: Option<usize>,
    logger: TsbundlerLogFn,
    logger_payload: *mut c_void,
}

// SpanHighlighter is not Send/Sync by default; we take a Mutex<> at the doc level
// when mutation is needed.  The ctx itself is accessed single-threaded from zedit.
unsafe impl Send for tsbundler_ctx_s {}
unsafe impl Sync for tsbundler_ctx_s {}

impl tsbundler_ctx_s {
    fn new() -> Option<Self> {
        let highlighter = SpanHighlighter::new().ok()?;
        let langs = build_lang_registry();
        let mut lang_lookup = HashMap::new();
        for (i, entry) in langs.iter().enumerate() {
            lang_lookup.insert(entry.canonical.to_string(), i);
        }

        let theme_names = available_themes();
        let mut themes = Vec::with_capacity(theme_names.len());
        let mut theme_lookup = HashMap::new();
        for &name in theme_names {
            match ActivatedTheme::from_name(name) {
                Ok(theme) => {
                    let i = themes.len();
                    theme_lookup.insert(name.to_string(), i);
                    themes.push(theme);
                }
                Err(_) => {} // skip broken themes
            }
        }

        Some(Self {
            highlighter,
            langs,
            lang_lookup,
            themes,
            theme_lookup,
            active_theme: None,
            logger: None,
            logger_payload: ptr::null_mut(),
        })
    }

    fn log(&self, log_type: i32, msg: &str) {
        if let Some(logger) = self.logger {
            if let Ok(c_msg) = CString::new(msg) {
                unsafe {
                    logger(self.logger_payload, log_type, c_msg.as_ptr());
                }
            }
        }
    }

    fn grammar_for_lang(&self, lang: &str) -> Option<Grammar> {
        Grammar::from_name(lang)
    }
}

// =============================================================================
// tsbundler_doc  (§8)
// =============================================================================

struct DocState {
    #[allow(dead_code)]
    grammar: Grammar,
    #[allow(dead_code)]
    lines_utf8: Vec<String>,
    /// Per-character attribute arrays from last parse (indexed by line, then char).
    last_attrs: Vec<Vec<ZeditAttr>>,
}

#[allow(non_camel_case_types)]
pub struct tsbundler_doc_s {
    ctx: *mut tsbundler_ctx_s,
    grammar: Grammar,
    state: Option<DocState>,
}

unsafe impl Send for tsbundler_doc_s {}

// =============================================================================
// Text ingestion: read UTF-16LE lines via callback → Vec<String> UTF-8
// =============================================================================

/// Read all lines from zedit's `read_fn` callback into UTF-8 strings.
/// `encoding` must be Utf16Le for zedit's UCS-2 buffers.
unsafe fn ingest_lines(
    read: TsbundlerReadFn,
    read_payload: *mut c_void,
    encoding: TsbundlerEncoding,
    total_lines: u32,
) -> Option<Vec<String>> {
    let read_fn = read?;
    let mut lines: Vec<String> = Vec::with_capacity(total_lines as usize);

    for row in 0..total_lines {
        let mut bytes_read: u32 = 0;
        // Request line at column 0 — tree-sitter wants the whole line.
        let ptr = read_fn(read_payload, 0, row, 0, &mut bytes_read);
        if ptr.is_null() || bytes_read == 0 {
            lines.push(String::new());
            continue;
        }
        let raw = slice::from_raw_parts(ptr as *const u8, bytes_read as usize);
        let line_text = match encoding {
            TsbundlerEncoding::Utf16Le => {
                // raw is UTF-16LE byte pairs
                let n_units = raw.len() / 2;
                let units: Vec<u16> = (0..n_units)
                    .map(|i| u16::from_le_bytes([raw[i * 2], raw[i * 2 + 1]]))
                    .collect();
                // Strip trailing newline/CR units
                let end = units
                    .iter()
                    .rposition(|&c| c != b'\n' as u16 && c != b'\r' as u16)
                    .map(|p| p + 1)
                    .unwrap_or(0);
                String::from_utf16_lossy(&units[..end]).into()
            }
            TsbundlerEncoding::Utf8 => {
                // Strip trailing newline bytes
                let end = raw
                    .iter()
                    .rposition(|&b| b != b'\n' && b != b'\r')
                    .map(|p| p + 1)
                    .unwrap_or(0);
                String::from_utf8_lossy(&raw[..end]).into_owned()
            }
            TsbundlerEncoding::Utf16Be => {
                let n_units = raw.len() / 2;
                let units: Vec<u16> = (0..n_units)
                    .map(|i| u16::from_be_bytes([raw[i * 2], raw[i * 2 + 1]]))
                    .collect();
                let end = units
                    .iter()
                    .rposition(|&c| c != b'\n' as u16 && c != b'\r' as u16)
                    .map(|p| p + 1)
                    .unwrap_or(0);
                String::from_utf16_lossy(&units[..end]).into()
            }
        };
        lines.push(line_text);
    }
    Some(lines)
}

// =============================================================================
// Highlight → per-line attr arrays
// =============================================================================

/// Build per-line per-character attr arrays from a `HighlightResult`.
/// `lines_utf8` is the per-line text (without newlines).
/// Returns `Vec<Vec<ZeditAttr>>` — one entry per line, one u16 per char.
fn highlight_to_line_attrs(
    result: &HighlightResult,
    theme: &ActivatedTheme,
    lines_utf8: &[String],
) -> Vec<Vec<ZeditAttr>> {
    // Build per-line byte-start table for the joined source (lines joined by '\n').
    let mut line_start: Vec<usize> = Vec::with_capacity(lines_utf8.len());
    let mut offset = 0usize;
    for line in lines_utf8 {
        line_start.push(offset);
        offset += line.len() + 1; // +1 for '\n' separator
    }

    // Allocate attr arrays (one ZeditAttr per Unicode codepoint, not per byte).
    let mut line_attrs: Vec<Vec<ZeditAttr>> = lines_utf8
        .iter()
        .map(|l| vec![ZEDIT_ATTR_NONE; l.chars().count()])
        .collect();

    for span in &result.spans {
        let capture_name = result
            .attrs
            .get(span.attr_id)
            .map(|a| a.capture_name.as_str())
            .unwrap_or("");
        let attr_val = theme.resolve_capture_index(capture_name);
        if attr_val == ZEDIT_ATTR_NONE {
            continue;
        }

        // Determine which lines and columns the span covers.
        let span_start = span.start_byte;
        let span_end = span.end_byte;

        for (row, (line, &ls)) in lines_utf8.iter().zip(line_start.iter()).enumerate() {
            let line_end_byte = ls + line.len(); // exclusive, before '\n'

            // Span must overlap this line's byte range.
            if span_end <= ls || span_start > line_end_byte {
                continue;
            }

            let local_start = span_start.saturating_sub(ls);
            let local_end = (span_end - ls).min(line.len());

            // Convert UTF-8 byte offsets to char positions.
            let char_start = line[..local_start.min(line.len())].chars().count();
            let char_end = line[..local_end].chars().count();

            if let Some(row_attrs) = line_attrs.get_mut(row) {
                for col in char_start..char_end.min(row_attrs.len()) {
                    row_attrs[col] = attr_val;
                }
            }
        }
    }

    line_attrs
}

/// Deliver `line_attrs` to zedit via the pull callback.
unsafe fn deliver_attrs(
    lines_utf8: &[String],
    line_attrs: &[Vec<ZeditAttr>],
    attrs_fn: TsbundlerAttrsFn,
    attrs_payload: *mut c_void,
) {
    let Some(cb) = attrs_fn else { return };
    for (row, (attrs, line)) in line_attrs.iter().zip(lines_utf8.iter()).enumerate() {
        let nchars = line.chars().count() as u32;
        if nchars == 0 {
            continue;
        }
        let buf = cb(attrs_payload, row as u32, nchars);
        if buf.is_null() {
            continue;
        }
        let out = slice::from_raw_parts_mut(buf, nchars as usize);
        out.copy_from_slice(&attrs[..nchars as usize]);
    }
}

// =============================================================================
// C API — library context (§4, §5, §6, §7)
// =============================================================================

#[no_mangle]
pub extern "C" fn tsbundler_init() -> *mut tsbundler_ctx_s {
    match tsbundler_ctx_s::new() {
        Some(ctx) => Box::into_raw(Box::new(ctx)),
        None => ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn tsbundler_free(ctx: *mut tsbundler_ctx_s) {
    if !ctx.is_null() {
        drop(Box::from_raw(ctx));
    }
}

/// §5 — install log relay.
#[no_mangle]
pub unsafe extern "C" fn tsbundler_set_logger(
    ctx: *mut tsbundler_ctx_s,
    logger: TsbundlerLogFn,
    payload: *mut c_void,
) {
    let Some(ctx) = ctx.as_mut() else { return };
    ctx.logger = logger;
    ctx.logger_payload = payload;
}

/// §6 — enumerate bundled grammars (callback-based).
#[no_mangle]
pub unsafe extern "C" fn tsbundler_enum_langs(
    ctx: *mut tsbundler_ctx_s,
    callback: TsbundlerLangFn,
    userdata: *mut c_void,
) -> ssize_t {
    let Some(ctx) = ctx.as_ref() else { return -1 };
    let count = ctx.langs.len() as ssize_t;
    let Some(cb) = callback else { return count };
    for entry in &ctx.langs {
        let ext_ptrs: Vec<*const c_char> = entry.c_extensions.iter().map(|e| e.as_ptr()).collect();
        cb(
            userdata,
            entry.c_name.as_ptr(),
            ext_ptrs.as_ptr(),
            entry.c_extensions.len() as c_int,
        );
    }
    count
}

/// §6 — check if a grammar is bundled.
#[no_mangle]
pub unsafe extern "C" fn tsbundler_supports_lang(
    ctx: *mut tsbundler_ctx_s,
    lang: *const c_char,
) -> c_int {
    let Some(ctx) = ctx.as_ref() else { return 0 };
    let Ok(lang) = (unsafe { CStr::from_ptr(lang) }).to_str() else {
        return 0;
    };
    ctx.grammar_for_lang(lang).is_some() as c_int
}

/// §7 — enumerate bundled themes (callback-based).
#[no_mangle]
pub unsafe extern "C" fn tsbundler_enum_themes(
    ctx: *mut tsbundler_ctx_s,
    callback: TsbundlerThemeFn,
    userdata: *mut c_void,
) -> ssize_t {
    let Some(ctx) = ctx.as_ref() else { return -1 };
    let count = ctx.themes.len() as ssize_t;
    let Some(cb) = callback else { return count };
    for theme in &ctx.themes {
        cb(userdata, theme.c_name.as_ptr());
    }
    count
}

/// §7 — activate a theme and enumerate its elements via callback.
/// Returns element count on success, negative TSBUNDLER_ERR_NOTHEME if not found.
#[no_mangle]
pub unsafe extern "C" fn tsbundler_theme_activate(
    ctx: *mut tsbundler_ctx_s,
    name: *const c_char,
    callback: TsbundlerElemFn,
    userdata: *mut c_void,
) -> ssize_t {
    let Some(ctx) = ctx.as_mut() else {
        return -(TsbundlerError::ErrInternal as i32) as ssize_t;
    };
    let Ok(name_str) = CStr::from_ptr(name).to_str() else {
        return -(TsbundlerError::ErrInternal as i32) as ssize_t;
    };
    let canonical = match canonical_theme_name(name_str) {
        Some(n) => n,
        None => return -(TsbundlerError::ErrNoTheme as i32) as ssize_t,
    };
    let Some(&idx) = ctx.theme_lookup.get(canonical) else {
        return -(TsbundlerError::ErrNoTheme as i32) as ssize_t;
    };
    ctx.active_theme = Some(idx);
    let theme = &ctx.themes[idx];
    let count = theme.elements.len() as ssize_t;
    let Some(cb) = callback else { return count };
    for elem in &theme.elements {
        cb(
            userdata,
            elem.name.as_ptr(),
            elem.fg,
            elem.bg,
            elem.attrs,
            elem.index as c_int,
        );
    }
    count
}

// =============================================================================
// C API — document (§8)
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn tsbundler_doc_create(
    ctx: *mut tsbundler_ctx_s,
    lang: *const c_char,
) -> *mut tsbundler_doc_s {
    let Some(ctx_ref) = ctx.as_ref() else {
        return ptr::null_mut();
    };
    let Ok(lang_str) = CStr::from_ptr(lang).to_str() else {
        return ptr::null_mut();
    };
    let Some(grammar) = ctx_ref.grammar_for_lang(lang_str) else {
        return ptr::null_mut();
    };
    Box::into_raw(Box::new(tsbundler_doc_s {
        ctx,
        grammar,
        state: None,
    }))
}

#[no_mangle]
pub unsafe extern "C" fn tsbundler_doc_free(doc: *mut tsbundler_doc_s) {
    if !doc.is_null() {
        drop(Box::from_raw(doc));
    }
}

/// §7.3 — full parse.
#[no_mangle]
pub unsafe extern "C" fn tsbundler_doc_parse_full(
    doc: *mut tsbundler_doc_s,
    read: TsbundlerReadFn,
    read_payload: *mut c_void,
    encoding: TsbundlerEncoding,
    total_lines: u32,
    attrs_fn: TsbundlerAttrsFn,
    attrs_payload: *mut c_void,
) -> TsbundlerError {
    let Some(doc) = doc.as_mut() else {
        return TsbundlerError::ErrInternal;
    };
    let Some(ctx) = doc.ctx.as_mut() else {
        return TsbundlerError::ErrInternal;
    };
    let Some(theme_idx) = ctx.active_theme else {
        ctx.log(2, "tsbundler_doc_parse_full: no active theme");
        return TsbundlerError::ErrNoTheme;
    };

    let Some(lines_utf8) = ingest_lines(read, read_payload, encoding, total_lines) else {
        return TsbundlerError::ErrInternal;
    };

    let source = lines_utf8.join("\n");
    let result = match ctx.highlighter.highlight(source.as_bytes(), doc.grammar) {
        Ok(r) => r,
        Err(e) => {
            ctx.log(2, &format!("highlight failed: {e}"));
            return TsbundlerError::ErrParse;
        }
    };

    let theme = &ctx.themes[theme_idx];
    let line_attrs = highlight_to_line_attrs(&result, theme, &lines_utf8);
    deliver_attrs(&lines_utf8, &line_attrs, attrs_fn, attrs_payload);

    doc.state = Some(DocState {
        grammar: doc.grammar,
        lines_utf8,
        last_attrs: line_attrs,
    });

    TsbundlerError::Ok
}

/// §7.4 — incremental edit + reparse.
///
/// For edit geometry, `edits` is provided for future incremental tree reuse.
/// Current implementation: full reparse, then diff attrs line-by-line to find
/// changed lines and fire `on_changed` only for those.
#[no_mangle]
pub unsafe extern "C" fn tsbundler_doc_edit(
    doc: *mut tsbundler_doc_s,
    _edits: *const TsbundlerEdit,
    _nedits: c_int,
    read: TsbundlerReadFn,
    read_payload: *mut c_void,
    encoding: TsbundlerEncoding,
    total_lines: u32,
    attrs_fn: TsbundlerAttrsFn,
    attrs_payload: *mut c_void,
    on_changed: TsbundlerRangeFn,
    on_changed_payload: *mut c_void,
) -> TsbundlerError {
    let Some(doc) = doc.as_mut() else {
        return TsbundlerError::ErrInternal;
    };
    let Some(ctx) = doc.ctx.as_mut() else {
        return TsbundlerError::ErrInternal;
    };
    let Some(theme_idx) = ctx.active_theme else {
        ctx.log(2, "tsbundler_doc_edit: no active theme");
        return TsbundlerError::ErrNoTheme;
    };

    let Some(lines_utf8) = ingest_lines(read, read_payload, encoding, total_lines) else {
        return TsbundlerError::ErrInternal;
    };

    let source = lines_utf8.join("\n");
    let result = match ctx.highlighter.highlight(source.as_bytes(), doc.grammar) {
        Ok(r) => r,
        Err(e) => {
            ctx.log(2, &format!("highlight failed: {e}"));
            return TsbundlerError::ErrParse;
        }
    };

    let theme = &ctx.themes[theme_idx];
    let new_attrs = highlight_to_line_attrs(&result, theme, &lines_utf8);
    deliver_attrs(&lines_utf8, &new_attrs, attrs_fn, attrs_payload);

    // Diff against previous attrs to fire on_changed for changed line ranges.
    if let Some(ref state) = doc.state {
        if let Some(cb) = on_changed {
            let old_attrs = &state.last_attrs;
            let n = new_attrs.len().max(old_attrs.len());
            let mut range_start: Option<u32> = None;
            for row in 0..n {
                let new_row = new_attrs.get(row).map(Vec::as_slice).unwrap_or(&[]);
                let old_row = old_attrs.get(row).map(Vec::as_slice).unwrap_or(&[]);
                let changed = new_row != old_row;
                match (changed, range_start) {
                    (true, None) => range_start = Some(row as u32),
                    (false, Some(start)) => {
                        cb(on_changed_payload, start, (row - 1) as u32);
                        range_start = None;
                    }
                    _ => {}
                }
            }
            if let Some(start) = range_start {
                cb(on_changed_payload, start, (n - 1) as u32);
            }
        }
    }

    doc.state = Some(DocState {
        grammar: doc.grammar,
        lines_utf8,
        last_attrs: new_attrs,
    });

    TsbundlerError::Ok
}

// =============================================================================
// recall_syntax_color_shim interface
//
// termio.c loads this dylib as "recall_syntax_color_shim" and calls:
//
//   int recall_ansi_color_render_range(
//       ucp  srcbuf,    // line as 8-bit chars (ASCII/latin-1)
//       int  linelen,   // full line length in chars
//       int  start,     // first char index to render
//       int  n,         // number of chars to render
//       ucp  outbuf,    // caller-allocated output buffer
//       int  outcap,    // output buffer capacity in bytes
//       int *pansilen   // set to bytes written on success
//   );
//   returns 1 on success, 0 on failure/fallback
// =============================================================================

struct RecallShimState {
    highlighter: SpanHighlighter,
    theme: theme_engine::theme_structures::Theme,
    cached_line: Vec<u8>,
    cached_spans: Vec<StyledSpan>,
    cached_lang: i32,
}

static RECALL_SHIM: Mutex<Option<RecallShimState>> = Mutex::new(None);

fn recall_shim_ensure_init() -> bool {
    let mut guard = match RECALL_SHIM.lock() {
        Ok(g) => g,
        Err(_) => return false,
    };
    if guard.is_some() {
        return true;
    }
    let highlighter = match SpanHighlighter::new() {
        Ok(h) => h,
        Err(_) => return false,
    };
    let theme = load_theme("tokyonight-dark").or_else(|_| load_theme("tokyonight-dark"));
    let theme = match theme {
        Ok(t) => t,
        Err(_) => return false,
    };
    *guard = Some(RecallShimState {
        highlighter,
        theme,
        cached_line: Vec::new(),
        cached_spans: Vec::new(),
        cached_lang: -1,
    });
    true
}

#[no_mangle]
pub unsafe extern "C" fn recall_ansi_color_render_range(
    srcbuf: *const c_uchar,
    linelen: c_int,
    start: c_int,
    n: c_int,
    outbuf: *mut c_uchar,
    outcap: c_int,
    lang_tag: c_int,
    pansilen: *mut c_int,
) -> c_int {
    if srcbuf.is_null() || outbuf.is_null() || pansilen.is_null() {
        return 0;
    }
    let linelen = linelen as usize;
    let start = start as usize;
    let n = n as usize;
    let outcap = outcap as usize;

    if linelen == 0 || n == 0 || start + n > linelen {
        return 0;
    }
    if !recall_shim_ensure_init() {
        return 0;
    }
    let mut guard = match RECALL_SHIM.lock() {
        Ok(g) => g,
        Err(_) => return 0,
    };
    let state = match guard.as_mut() {
        Some(s) => s,
        None => return 0,
    };

    let line = slice::from_raw_parts(srcbuf, linelen);

    // Only re-parse if the line content or grammar changed since last call.
    if state.cached_line != line || state.cached_lang != lang_tag {
        let grammar = match lang_tag {
            1 => Grammar::Sql,
            2 => Grammar::Python,
            3 => Grammar::Mdx,
            _ => Grammar::ObjectScript,
        };
        let highlight = match state.highlighter.highlight(line, grammar) {
            Ok(h) => h,
            Err(_) => return 0,
        };
        let styled = match resolve_styled_spans_for_source(line.len(), &highlight, &state.theme) {
            Ok(s) => s,
            Err(_) => return 0,
        };
        state.cached_line = line.to_vec();
        state.cached_spans = styled;
        state.cached_lang = lang_tag;
    }

    // Clip spans to the requested [start..start+n] subrange.
    let end = start + n;
    let sub_source = &line[start..end];
    let sub_spans: Vec<StyledSpan> = state
        .cached_spans
        .iter()
        .filter(|s| s.end_byte > start && s.start_byte < end)
        .map(|s| StyledSpan {
            start_byte: s.start_byte.saturating_sub(start).min(n),
            end_byte: s.end_byte.saturating_sub(start).min(n),
            style: s.style,
        })
        .filter(|s| s.start_byte < s.end_byte)
        .collect();

    // Full (non-diff) ANSI render of the sub-range with reset at the end.
    let rendered = match render_ansi(sub_source, &sub_spans, ColorMode::TrueColor, false) {
        Ok(r) => r,
        Err(_) => return 0,
    };

    if rendered.is_empty() {
        *pansilen = 0;
        return 1;
    }

    // Append SGR reset so subsequent plain text isn't colored.
    let with_reset = format!("{}\x1b[0m", rendered);
    let out_bytes = with_reset.as_bytes();

    if out_bytes.len() > outcap {
        // Signal needed size; C caller will grow the buffer and retry.
        *pansilen = out_bytes.len() as c_int;
        return 1;
    }

    ptr::copy_nonoverlapping(out_bytes.as_ptr(), outbuf, out_bytes.len());
    *pansilen = out_bytes.len() as c_int;
    1
}
