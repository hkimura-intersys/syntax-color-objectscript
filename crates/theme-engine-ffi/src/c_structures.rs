use highlight_spans::highlight_structures::SpanHighlighter;
use render_ansi::ansi_structures::{IncrementalRenderer, StreamLineRenderer};
use theme_engine::theme_structures::Theme;

/// Operation succeeded.
pub const SYNTAX_COLOR_FFI_OK: i32 = 0;
/// A required pointer argument was null.
pub const SYNTAX_COLOR_FFI_ERR_NULL: i32 = 1;
/// A C string argument was not valid UTF-8.
pub const SYNTAX_COLOR_FFI_ERR_UTF8: i32 = 2;
/// Theme loading or parsing failed.
pub const SYNTAX_COLOR_FFI_ERR_THEME: i32 = 3;
/// Requested style was not found.
pub const SYNTAX_COLOR_FFI_ERR_NOT_FOUND: i32 = 4;
/// Highlighting or highlighter initialization failed.
pub const SYNTAX_COLOR_FFI_ERR_HIGHLIGHT: i32 = 5;
/// Rendering or span resolution failed.
pub const SYNTAX_COLOR_FFI_ERR_RENDER: i32 = 6;
/// An enum value or argument payload was invalid.
pub const SYNTAX_COLOR_FFI_ERR_INVALID_ARGUMENT: i32 = 7;

/// Backward-compatible aliases for the original theme-only FFI constants.
pub const THEME_ENGINE_FFI_OK: i32 = SYNTAX_COLOR_FFI_OK;
pub const THEME_ENGINE_FFI_ERR_NULL: i32 = SYNTAX_COLOR_FFI_ERR_NULL;
pub const THEME_ENGINE_FFI_ERR_UTF8: i32 = SYNTAX_COLOR_FFI_ERR_UTF8;
pub const THEME_ENGINE_FFI_ERR_THEME: i32 = SYNTAX_COLOR_FFI_ERR_THEME;
pub const THEME_ENGINE_FFI_ERR_NOT_FOUND: i32 = SYNTAX_COLOR_FFI_ERR_NOT_FOUND;

/// Opaque C handle for a loaded theme.
pub struct ThemeEngineTheme {
    pub(crate) theme: Theme,
}

/// Opaque C handle for a reusable Tree-sitter highlighter.
pub struct SyntaxColorHighlighter {
    pub(crate) highlighter: SpanHighlighter,
}

/// Opaque C handle for a multi-line incremental VT patch renderer.
pub struct SyntaxColorIncrementalRenderer {
    pub(crate) renderer: IncrementalRenderer,
}

/// Opaque C handle for a single-line relative VT patch renderer.
pub struct SyntaxColorStreamLineRenderer {
    pub(crate) renderer: StreamLineRenderer,
}

/// C ABI RGB triplet.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct ThemeEngineRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// C ABI style payload.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct ThemeEngineStyle {
    pub has_fg: u8,
    pub fg: ThemeEngineRgb,
    pub has_bg: u8,
    pub bg: ThemeEngineRgb,
    pub bold: u8,
    pub italic: u8,
    pub underline: u8,
}

/// Owned byte buffer returned from the FFI.
#[repr(C)]
#[derive(Debug, Default)]
pub struct SyntaxColorString {
    pub data: *mut u8,
    pub len: usize,
}

/// Owned array of byte buffers returned from the FFI.
#[repr(C)]
#[derive(Debug, Default)]
pub struct SyntaxColorStringArray {
    pub items: *mut SyntaxColorString,
    pub count: usize,
}

/// Highlight attribute entry: `id -> capture_name`.
#[repr(C)]
#[derive(Debug, Default)]
pub struct SyntaxColorAttr {
    pub id: usize,
    pub capture_name: SyntaxColorString,
}

/// Highlight span entry using byte offsets into the original source.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct SyntaxColorSpan {
    pub attr_id: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Owned highlight output buffer.
#[repr(C)]
#[derive(Debug, Default)]
pub struct SyntaxColorHighlightResult {
    pub attrs: *mut SyntaxColorAttr,
    pub attr_count: usize,
    pub spans: *mut SyntaxColorSpan,
    pub span_count: usize,
}

/// Render-ready styled span entry.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct SyntaxColorStyledSpan {
    pub start_byte: usize,
    pub end_byte: usize,
    pub has_style: u8,
    pub style: ThemeEngineStyle,
}

/// Owned styled span buffer.
#[repr(C)]
#[derive(Debug, Default)]
pub struct SyntaxColorStyledSpanBuffer {
    pub spans: *mut SyntaxColorStyledSpan,
    pub count: usize,
}
