use highlight_spans::highlight_structures::HighlightError;
use theme_engine::theme_structures::Style;
use thiserror::Error;

pub const CSI: &str = "\x1b[";
pub const SGR_RESET: &str = "\x1b[0m";
pub const EL_TO_END: &str = "\x1b[K";
pub const OSC: &str = "\x1b]";
pub const ST_BEL: &str = "\x07";
pub const OSC_RESET_DEFAULT_FG: &str = "\x1b]110\x07";
pub const OSC_RESET_DEFAULT_BG: &str = "\x1b]111\x07";
pub const TAB_STOP: usize = 8;
pub const ANSI_256_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
pub const COLOR_MODE_NAMES: [&str; 3] = ["truecolor", "ansi256", "ansi16"];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum ColorMode {
    #[default]
    TrueColor,
    Ansi256,
    Ansi16,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StyledSpan {
    pub start_byte: usize,
    pub end_byte: usize,
    pub style: Option<Style>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StyledCell {
    pub(crate) text: String,
    pub(crate) style: Option<Style>,
    pub(crate) width: usize,
}

#[derive(Debug, Clone)]
pub struct IncrementalRenderer {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) origin_row: usize,
    pub(crate) origin_col: usize,
    pub(crate) color_mode: ColorMode,
    pub(crate) preserve_terminal_background: bool,
    pub(crate) prev_lines: Vec<Vec<StyledCell>>,
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("highlighting failed: {0}")]
    Highlight(#[from] HighlightError),
    #[error("invalid span range {start_byte}..{end_byte} for source length {source_len}")]
    SpanOutOfBounds {
        start_byte: usize,
        end_byte: usize,
        source_len: usize,
    },
    #[error(
        "spans must be sorted and non-overlapping: prev_end={prev_end}, next_start={next_start}"
    )]
    OverlappingSpans { prev_end: usize, next_start: usize },
    #[error("invalid attr_id {attr_id}; attrs length is {attrs_len}")]
    InvalidAttrId { attr_id: usize, attrs_len: usize },
    #[error("stream line patch requires single-line input without newlines")]
    MultiLineInput,
}

/// Incremental renderer for a single mutable line without terminal width assumptions.
///
/// This renderer avoids absolute cursor positioning. It assumes each emitted
/// patch is written to the same terminal line and the cursor remains at the end
/// of the previously rendered line.
#[derive(Debug, Clone)]
pub struct StreamLineRenderer {
    pub color_mode: ColorMode,
    pub preserve_terminal_background: bool,
    pub prev_line: Vec<StyledCell>,
}
