use std::fmt::Write;

use highlight_spans::{Grammar, HighlightError, HighlightResult, SpanHighlighter};
use theme_engine::{Rgb, Style, Theme};
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const CSI: &str = "\x1b[";
const SGR_RESET: &str = "\x1b[0m";
const EL_TO_END: &str = "\x1b[K";
const OSC: &str = "\x1b]";
const ST_BEL: &str = "\x07";
const OSC_RESET_DEFAULT_FG: &str = "\x1b]110\x07";
const OSC_RESET_DEFAULT_BG: &str = "\x1b]111\x07";
const TAB_STOP: usize = 8;
const ANSI_256_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
const COLOR_MODE_NAMES: [&str; 3] = ["truecolor", "ansi256", "ansi16"];
const PRESERVE_TERMINAL_BACKGROUND_DEFAULT: bool = true;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum ColorMode {
    #[default]
    TrueColor,
    Ansi256,
    Ansi16,
}

impl ColorMode {
    /// Parses a color mode from user input.
    ///
    /// Accepts `"truecolor"`, `"24bit"`, `"24-bit"`, `"ansi256"`, `"256"`,
    /// `"ansi16"`, and `"16"`.
    #[must_use]
    pub fn from_name(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "truecolor" | "24bit" | "24-bit" | "rgb" => Some(Self::TrueColor),
            "ansi256" | "256" | "xterm256" | "xterm-256" => Some(Self::Ansi256),
            "ansi16" | "16" | "xterm16" | "xterm-16" | "basic" => Some(Self::Ansi16),
            _ => None,
        }
    }

    /// Returns the canonical CLI/config name for this mode.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::TrueColor => "truecolor",
            Self::Ansi256 => "ansi256",
            Self::Ansi16 => "ansi16",
        }
    }

    /// Returns all canonical color mode names.
    #[must_use]
    pub const fn supported_names() -> &'static [&'static str] {
        &COLOR_MODE_NAMES
    }
}

/// Builds OSC `10` to set terminal default foreground color.
#[must_use]
pub fn osc_set_default_foreground(color: Rgb) -> String {
    format!(
        "{OSC}10;#{:02x}{:02x}{:02x}{ST_BEL}",
        color.r, color.g, color.b
    )
}

/// Builds OSC `11` to set terminal default background color.
#[must_use]
pub fn osc_set_default_background(color: Rgb) -> String {
    format!(
        "{OSC}11;#{:02x}{:02x}{:02x}{ST_BEL}",
        color.r, color.g, color.b
    )
}

/// Builds OSC `10`/`11` terminal default-color updates.
///
/// Omitted channels are not emitted.
#[must_use]
pub fn osc_set_default_colors(fg: Option<Rgb>, bg: Option<Rgb>) -> String {
    let mut out = String::new();
    if let Some(color) = fg {
        out.push_str(&osc_set_default_foreground(color));
    }
    if let Some(color) = bg {
        out.push_str(&osc_set_default_background(color));
    }
    out
}

/// Builds OSC terminal default-color updates from a theme.
///
/// Colors are resolved from `default_fg`/`default_bg` UI roles first, then
/// fall back to the theme `normal` style.
#[must_use]
pub fn osc_set_default_colors_from_theme(theme: &Theme) -> String {
    let (fg, bg) = theme.default_terminal_colors();
    osc_set_default_colors(fg, bg)
}

/// Returns OSC `110` to reset terminal default foreground color.
#[must_use]
pub const fn osc_reset_default_foreground() -> &'static str {
    OSC_RESET_DEFAULT_FG
}

/// Returns OSC `111` to reset terminal default background color.
#[must_use]
pub const fn osc_reset_default_background() -> &'static str {
    OSC_RESET_DEFAULT_BG
}

/// Returns OSC `110` + `111` to reset terminal default foreground/background colors.
#[must_use]
pub const fn osc_reset_default_colors() -> &'static str {
    "\x1b]110\x07\x1b]111\x07"
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StyledSpan {
    pub start_byte: usize,
    pub end_byte: usize,
    pub style: Option<Style>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct StyledCell {
    text: String,
    style: Option<Style>,
    width: usize,
}

#[derive(Debug, Clone)]
pub struct IncrementalRenderer {
    width: usize,
    height: usize,
    origin_row: usize,
    origin_col: usize,
    color_mode: ColorMode,
    preserve_terminal_background: bool,
    prev_lines: Vec<Vec<StyledCell>>,
}

impl IncrementalRenderer {
    /// Creates an incremental renderer with a bounded viewport size.
    ///
    /// A minimum viewport size of `1x1` is enforced.
    /// The render origin defaults to terminal row `1`, column `1`.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width: width.max(1),
            height: height.max(1),
            origin_row: 1,
            origin_col: 1,
            color_mode: ColorMode::TrueColor,
            preserve_terminal_background: PRESERVE_TERMINAL_BACKGROUND_DEFAULT,
            prev_lines: Vec::new(),
        }
    }

    /// Resizes the viewport and clips cached state to the new bounds.
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.prev_lines = clip_lines_to_viewport(&self.prev_lines, self.width, self.height);
    }

    /// Clears all cached frame state for this renderer.
    pub fn clear_state(&mut self) {
        self.prev_lines.clear();
    }

    /// Sets the terminal origin used for generated CUP cursor positions.
    ///
    /// The origin is 1-based terminal coordinates (`row`, `col`) in display cells.
    /// Values lower than `1` are clamped to `1`.
    pub fn set_origin(&mut self, row: usize, col: usize) {
        self.origin_row = row.max(1);
        self.origin_col = col.max(1);
    }

    /// Returns the current 1-based terminal origin (`row`, `col`).
    #[must_use]
    pub fn origin(&self) -> (usize, usize) {
        (self.origin_row, self.origin_col)
    }

    /// Sets the ANSI color mode used by this renderer.
    pub fn set_color_mode(&mut self, color_mode: ColorMode) {
        self.color_mode = color_mode;
    }

    /// Returns the current ANSI color mode.
    #[must_use]
    pub fn color_mode(&self) -> ColorMode {
        self.color_mode
    }

    /// Controls whether ANSI rendering preserves the terminal's existing background.
    ///
    /// When set to `true` (default), background colors from the theme are ignored.
    /// When set to `false`, background colors from resolved theme styles are emitted.
    pub fn set_preserve_terminal_background(&mut self, preserve_terminal_background: bool) {
        self.preserve_terminal_background = preserve_terminal_background;
    }

    /// Returns whether terminal background passthrough is enabled.
    #[must_use]
    pub fn preserve_terminal_background(&self) -> bool {
        self.preserve_terminal_background
    }

    /// Renders only the VT patch from the cached frame to `source`.
    ///
    /// The method validates input spans, projects them to styled terminal cells,
    /// diffs against previous state, and returns only changed cursor/style output.
    ///
    /// # Errors
    ///
    /// Returns an error when spans are out of bounds, unsorted, or overlapping.
    pub fn render_patch(
        &mut self,
        source: &[u8],
        spans: &[StyledSpan],
    ) -> Result<String, RenderError> {
        validate_spans(source.len(), spans)?;
        let curr_lines = build_styled_cells(source, spans, self.width, self.height);
        let patch = diff_lines_to_patch(
            &self.prev_lines,
            &curr_lines,
            self.origin_row,
            self.origin_col,
            self.color_mode,
            self.preserve_terminal_background,
        );
        self.prev_lines = curr_lines;
        Ok(patch)
    }

    /// Runs highlight + theme resolution + incremental diff in one call.
    ///
    /// # Errors
    ///
    /// Returns an error if highlighting fails or spans fail validation.
    pub fn highlight_to_patch(
        &mut self,
        highlighter: &mut SpanHighlighter,
        source: &[u8],
        flavor: Grammar,
        theme: &Theme,
    ) -> Result<String, RenderError> {
        let highlight = highlighter.highlight(source, flavor)?;
        let styled = resolve_styled_spans_for_source(source.len(), &highlight, theme)?;
        self.render_patch(source, &styled)
    }
}

/// Incremental renderer for a single mutable line without terminal width assumptions.
///
/// This renderer avoids absolute cursor positioning. It assumes each emitted
/// patch is written to the same terminal line and the cursor remains at the end
/// of the previously rendered line.
#[derive(Debug, Clone)]
pub struct StreamLineRenderer {
    color_mode: ColorMode,
    preserve_terminal_background: bool,
    prev_line: Vec<StyledCell>,
}

impl StreamLineRenderer {
    /// Creates a line renderer with truecolor output.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears prior line state.
    pub fn clear_state(&mut self) {
        self.prev_line.clear();
    }

    /// Sets the ANSI color mode used by this renderer.
    pub fn set_color_mode(&mut self, color_mode: ColorMode) {
        self.color_mode = color_mode;
    }

    /// Returns the current ANSI color mode.
    #[must_use]
    pub fn color_mode(&self) -> ColorMode {
        self.color_mode
    }

    /// Controls whether ANSI rendering preserves the terminal's existing background.
    ///
    /// When set to `true` (default), background colors from the theme are ignored.
    /// When set to `false`, background colors from resolved theme styles are emitted.
    pub fn set_preserve_terminal_background(&mut self, preserve_terminal_background: bool) {
        self.preserve_terminal_background = preserve_terminal_background;
    }

    /// Returns whether terminal background passthrough is enabled.
    #[must_use]
    pub fn preserve_terminal_background(&self) -> bool {
        self.preserve_terminal_background
    }

    /// Renders a width-independent patch for a single line.
    ///
    /// # Errors
    ///
    /// Returns an error when spans are invalid or input contains a newline.
    pub fn render_line_patch(
        &mut self,
        source: &[u8],
        spans: &[StyledSpan],
    ) -> Result<String, RenderError> {
        validate_spans(source.len(), spans)?;
        if source.contains(&b'\n') {
            return Err(RenderError::MultiLineInput);
        }

        let curr_line = build_styled_line_cells(source, spans);
        let patch = diff_single_line_to_patch(
            &self.prev_line,
            &curr_line,
            self.color_mode,
            self.preserve_terminal_background,
        );
        self.prev_line = curr_line;
        Ok(patch)
    }

    /// Runs highlight + theme resolution + stream-safe single-line diff.
    ///
    /// # Errors
    ///
    /// Returns an error if highlighting fails, spans are invalid, or input has newlines.
    pub fn highlight_line_to_patch(
        &mut self,
        highlighter: &mut SpanHighlighter,
        source: &[u8],
        flavor: Grammar,
        theme: &Theme,
    ) -> Result<String, RenderError> {
        let highlight = highlighter.highlight(source, flavor)?;
        let styled = resolve_styled_spans_for_source(source.len(), &highlight, theme)?;
        self.render_line_patch(source, &styled)
    }
}

impl Default for StreamLineRenderer {
    fn default() -> Self {
        Self {
            color_mode: ColorMode::TrueColor,
            preserve_terminal_background: PRESERVE_TERMINAL_BACKGROUND_DEFAULT,
            prev_line: Vec::new(),
        }
    }
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

/// Resolves highlight spans into renderable spans by attaching theme styles.
///
/// Resolved capture styles are layered over the theme `normal` style, so token
/// styles inherit unspecified fields (for example background color).
///
/// # Errors
///
/// Returns [`RenderError::InvalidAttrId`] when a span references a missing attribute.
pub fn resolve_styled_spans(
    highlight: &HighlightResult,
    theme: &Theme,
) -> Result<Vec<StyledSpan>, RenderError> {
    let normal_style = theme.get_exact("normal").copied();
    let mut out = Vec::with_capacity(highlight.spans.len());
    for span in &highlight.spans {
        let Some(attr) = highlight.attrs.get(span.attr_id) else {
            return Err(RenderError::InvalidAttrId {
                attr_id: span.attr_id,
                attrs_len: highlight.attrs.len(),
            });
        };
        let capture_style = theme.resolve(&attr.capture_name).copied();
        out.push(StyledSpan {
            start_byte: span.start_byte,
            end_byte: span.end_byte,
            style: merge_styles(normal_style, capture_style),
        });
    }
    Ok(out)
}

/// Resolves styled spans and fills uncovered byte ranges with `normal` style.
fn resolve_styled_spans_for_source(
    source_len: usize,
    highlight: &HighlightResult,
    theme: &Theme,
) -> Result<Vec<StyledSpan>, RenderError> {
    let spans = resolve_styled_spans(highlight, theme)?;
    Ok(fill_uncovered_ranges_with_style(
        source_len,
        spans,
        theme.get_exact("normal").copied(),
    ))
}

/// Merges an overlay style over a base style.
///
/// Color fields in `overlay` replace the base when present. Boolean attributes
/// are merged with logical OR.
fn merge_styles(base: Option<Style>, overlay: Option<Style>) -> Option<Style> {
    match (base, overlay) {
        (None, None) => None,
        (Some(base), None) => Some(base),
        (None, Some(overlay)) => Some(overlay),
        (Some(base), Some(overlay)) => Some(Style {
            fg: overlay.fg.or(base.fg),
            bg: overlay.bg.or(base.bg),
            bold: base.bold || overlay.bold,
            italic: base.italic || overlay.italic,
            underline: base.underline || overlay.underline,
        }),
    }
}

/// Inserts default-style spans for byte ranges not covered by highlight spans.
fn fill_uncovered_ranges_with_style(
    source_len: usize,
    spans: Vec<StyledSpan>,
    default_style: Option<Style>,
) -> Vec<StyledSpan> {
    let Some(default_style) = default_style else {
        return spans;
    };

    let mut out = Vec::with_capacity(spans.len().saturating_mul(2).saturating_add(1));
    let mut cursor = 0usize;
    for span in spans {
        if cursor < span.start_byte {
            out.push(StyledSpan {
                start_byte: cursor,
                end_byte: span.start_byte,
                style: Some(default_style),
            });
        }

        if span.start_byte < span.end_byte {
            out.push(span);
        }
        cursor = cursor.max(span.end_byte);
    }

    if cursor < source_len {
        out.push(StyledSpan {
            start_byte: cursor,
            end_byte: source_len,
            style: Some(default_style),
        });
    }

    out
}

/// Renders a source buffer and styled spans into a single ANSI string.
///
/// # Errors
///
/// Returns an error when spans are out of bounds, unsorted, or overlapping.
pub fn render_ansi(source: &[u8], spans: &[StyledSpan]) -> Result<String, RenderError> {
    render_ansi_with_mode(source, spans, ColorMode::TrueColor)
}

/// Renders a source buffer and styled spans into a single ANSI string.
///
/// # Errors
///
/// Returns an error when spans are out of bounds, unsorted, or overlapping.
pub fn render_ansi_with_mode(
    source: &[u8],
    spans: &[StyledSpan],
    color_mode: ColorMode,
) -> Result<String, RenderError> {
    render_ansi_with_mode_and_background(
        source,
        spans,
        color_mode,
        PRESERVE_TERMINAL_BACKGROUND_DEFAULT,
    )
}

/// Renders a source buffer and styled spans into a single ANSI string.
///
/// When `preserve_terminal_background` is `true`, background colors in styles
/// are ignored so output keeps the terminal's current background.
///
/// # Errors
///
/// Returns an error when spans are out of bounds, unsorted, or overlapping.
pub fn render_ansi_with_mode_and_background(
    source: &[u8],
    spans: &[StyledSpan],
    color_mode: ColorMode,
    preserve_terminal_background: bool,
) -> Result<String, RenderError> {
    validate_spans(source.len(), spans)?;

    let mut out = String::new();
    let mut cursor = 0usize;
    for span in spans {
        if cursor < span.start_byte {
            out.push_str(&String::from_utf8_lossy(&source[cursor..span.start_byte]));
        }
        append_styled_segment(
            &mut out,
            &source[span.start_byte..span.end_byte],
            span.style,
            color_mode,
            preserve_terminal_background,
        );
        cursor = span.end_byte;
    }

    if cursor < source.len() {
        out.push_str(&String::from_utf8_lossy(&source[cursor..]));
    }

    Ok(out)
}

/// Renders a source buffer and styled spans into per-line ANSI strings.
///
/// Spans that cross line boundaries are clipped per line.
///
/// # Errors
///
/// Returns an error when spans are out of bounds, unsorted, or overlapping.
pub fn render_ansi_lines(source: &[u8], spans: &[StyledSpan]) -> Result<Vec<String>, RenderError> {
    render_ansi_lines_with_mode(source, spans, ColorMode::TrueColor)
}

/// Renders a source buffer and styled spans into per-line ANSI strings.
///
/// Spans that cross line boundaries are clipped per line.
///
/// # Errors
///
/// Returns an error when spans are out of bounds, unsorted, or overlapping.
pub fn render_ansi_lines_with_mode(
    source: &[u8],
    spans: &[StyledSpan],
    color_mode: ColorMode,
) -> Result<Vec<String>, RenderError> {
    render_ansi_lines_with_mode_and_background(
        source,
        spans,
        color_mode,
        PRESERVE_TERMINAL_BACKGROUND_DEFAULT,
    )
}

/// Renders a source buffer and styled spans into per-line ANSI strings.
///
/// Spans that cross line boundaries are clipped per line.
/// When `preserve_terminal_background` is `true`, background colors in styles
/// are ignored so output keeps the terminal's current background.
///
/// # Errors
///
/// Returns an error when spans are out of bounds, unsorted, or overlapping.
pub fn render_ansi_lines_with_mode_and_background(
    source: &[u8],
    spans: &[StyledSpan],
    color_mode: ColorMode,
    preserve_terminal_background: bool,
) -> Result<Vec<String>, RenderError> {
    validate_spans(source.len(), spans)?;

    let line_ranges = compute_line_ranges(source);
    let mut lines = Vec::with_capacity(line_ranges.len());
    let mut span_cursor = 0usize;

    for (line_start, line_end) in line_ranges {
        while span_cursor < spans.len() && spans[span_cursor].end_byte <= line_start {
            span_cursor += 1;
        }

        let mut line = String::new();
        let mut cursor = line_start;
        let mut i = span_cursor;
        while i < spans.len() {
            let span = spans[i];
            if span.start_byte >= line_end {
                break;
            }

            let seg_start = span.start_byte.max(line_start);
            let seg_end = span.end_byte.min(line_end);
            if cursor < seg_start {
                line.push_str(&String::from_utf8_lossy(&source[cursor..seg_start]));
            }
            append_styled_segment(
                &mut line,
                &source[seg_start..seg_end],
                span.style,
                color_mode,
                preserve_terminal_background,
            );
            cursor = seg_end;
            i += 1;
        }

        if cursor < line_end {
            line.push_str(&String::from_utf8_lossy(&source[cursor..line_end]));
        }

        lines.push(line);
    }

    Ok(lines)
}

/// Highlights and renders a source buffer to ANSI output.
///
/// This convenience API creates a temporary [`SpanHighlighter`].
///
/// # Errors
///
/// Returns an error if highlighting fails or if rendered spans are invalid.
pub fn highlight_to_ansi(
    source: &[u8],
    flavor: Grammar,
    theme: &Theme,
) -> Result<String, RenderError> {
    let mut highlighter = SpanHighlighter::new()?;
    highlight_to_ansi_with_highlighter_and_mode(
        &mut highlighter,
        source,
        flavor,
        theme,
        ColorMode::TrueColor,
    )
}

/// Highlights and renders a source buffer using a caller-provided highlighter.
///
/// # Errors
///
/// Returns an error if highlighting fails or if rendered spans are invalid.
pub fn highlight_to_ansi_with_highlighter(
    highlighter: &mut SpanHighlighter,
    source: &[u8],
    flavor: Grammar,
    theme: &Theme,
) -> Result<String, RenderError> {
    highlight_to_ansi_with_highlighter_and_mode(
        highlighter,
        source,
        flavor,
        theme,
        ColorMode::TrueColor,
    )
}

/// Highlights and renders a source buffer with an explicit ANSI color mode.
///
/// This convenience API creates a temporary [`SpanHighlighter`].
///
/// # Errors
///
/// Returns an error if highlighting fails or if rendered spans are invalid.
pub fn highlight_to_ansi_with_mode(
    source: &[u8],
    flavor: Grammar,
    theme: &Theme,
    color_mode: ColorMode,
) -> Result<String, RenderError> {
    highlight_to_ansi_with_mode_and_background(
        source,
        flavor,
        theme,
        color_mode,
        PRESERVE_TERMINAL_BACKGROUND_DEFAULT,
    )
}

/// Highlights and renders a source buffer with explicit color and background behavior.
///
/// # Errors
///
/// Returns an error if highlighting fails or if rendered spans are invalid.
pub fn highlight_to_ansi_with_mode_and_background(
    source: &[u8],
    flavor: Grammar,
    theme: &Theme,
    color_mode: ColorMode,
    preserve_terminal_background: bool,
) -> Result<String, RenderError> {
    let mut highlighter = SpanHighlighter::new()?;
    highlight_to_ansi_with_highlighter_and_mode_and_background(
        &mut highlighter,
        source,
        flavor,
        theme,
        color_mode,
        preserve_terminal_background,
    )
}

/// Highlights and renders a source buffer using a caller-provided highlighter and color mode.
///
/// # Errors
///
/// Returns an error if highlighting fails or if rendered spans are invalid.
pub fn highlight_to_ansi_with_highlighter_and_mode(
    highlighter: &mut SpanHighlighter,
    source: &[u8],
    flavor: Grammar,
    theme: &Theme,
    color_mode: ColorMode,
) -> Result<String, RenderError> {
    highlight_to_ansi_with_highlighter_and_mode_and_background(
        highlighter,
        source,
        flavor,
        theme,
        color_mode,
        PRESERVE_TERMINAL_BACKGROUND_DEFAULT,
    )
}

/// Highlights and renders a source buffer using caller-provided highlighter and explicit background behavior.
///
/// # Errors
///
/// Returns an error if highlighting fails or if rendered spans are invalid.
pub fn highlight_to_ansi_with_highlighter_and_mode_and_background(
    highlighter: &mut SpanHighlighter,
    source: &[u8],
    flavor: Grammar,
    theme: &Theme,
    color_mode: ColorMode,
    preserve_terminal_background: bool,
) -> Result<String, RenderError> {
    let highlight = highlighter.highlight(source, flavor)?;
    let styled = resolve_styled_spans_for_source(source.len(), &highlight, theme)?;
    render_ansi_with_mode_and_background(source, &styled, color_mode, preserve_terminal_background)
}

/// Highlights line-oriented input and returns ANSI output per line.
///
/// This convenience API creates a temporary [`SpanHighlighter`].
///
/// # Errors
///
/// Returns an error if highlighting fails or if rendered spans are invalid.
pub fn highlight_lines_to_ansi_lines<S: AsRef<str>>(
    lines: &[S],
    flavor: Grammar,
    theme: &Theme,
) -> Result<Vec<String>, RenderError> {
    let mut highlighter = SpanHighlighter::new()?;
    highlight_lines_to_ansi_lines_with_highlighter_and_mode(
        &mut highlighter,
        lines,
        flavor,
        theme,
        ColorMode::TrueColor,
    )
}

/// Highlights line-oriented input with a caller-provided highlighter.
///
/// # Errors
///
/// Returns an error if highlighting fails or if rendered spans are invalid.
pub fn highlight_lines_to_ansi_lines_with_highlighter<S: AsRef<str>>(
    highlighter: &mut SpanHighlighter,
    lines: &[S],
    flavor: Grammar,
    theme: &Theme,
) -> Result<Vec<String>, RenderError> {
    highlight_lines_to_ansi_lines_with_highlighter_and_mode(
        highlighter,
        lines,
        flavor,
        theme,
        ColorMode::TrueColor,
    )
}

/// Highlights line-oriented input and returns ANSI output per line using a color mode.
///
/// This convenience API creates a temporary [`SpanHighlighter`].
///
/// # Errors
///
/// Returns an error if highlighting fails or if rendered spans are invalid.
pub fn highlight_lines_to_ansi_lines_with_mode<S: AsRef<str>>(
    lines: &[S],
    flavor: Grammar,
    theme: &Theme,
    color_mode: ColorMode,
) -> Result<Vec<String>, RenderError> {
    highlight_lines_to_ansi_lines_with_mode_and_background(
        lines,
        flavor,
        theme,
        color_mode,
        PRESERVE_TERMINAL_BACKGROUND_DEFAULT,
    )
}

/// Highlights line-oriented input and returns ANSI output per line with explicit color and background behavior.
///
/// This convenience API creates a temporary [`SpanHighlighter`].
///
/// # Errors
///
/// Returns an error if highlighting fails or if rendered spans are invalid.
pub fn highlight_lines_to_ansi_lines_with_mode_and_background<S: AsRef<str>>(
    lines: &[S],
    flavor: Grammar,
    theme: &Theme,
    color_mode: ColorMode,
    preserve_terminal_background: bool,
) -> Result<Vec<String>, RenderError> {
    let mut highlighter = SpanHighlighter::new()?;
    highlight_lines_to_ansi_lines_with_highlighter_and_mode_and_background(
        &mut highlighter,
        lines,
        flavor,
        theme,
        color_mode,
        preserve_terminal_background,
    )
}

/// Highlights line-oriented input with a caller-provided highlighter and color mode.
///
/// # Errors
///
/// Returns an error if highlighting fails or if rendered spans are invalid.
pub fn highlight_lines_to_ansi_lines_with_highlighter_and_mode<S: AsRef<str>>(
    highlighter: &mut SpanHighlighter,
    lines: &[S],
    flavor: Grammar,
    theme: &Theme,
    color_mode: ColorMode,
) -> Result<Vec<String>, RenderError> {
    highlight_lines_to_ansi_lines_with_highlighter_and_mode_and_background(
        highlighter,
        lines,
        flavor,
        theme,
        color_mode,
        PRESERVE_TERMINAL_BACKGROUND_DEFAULT,
    )
}

/// Highlights line-oriented input with caller-provided highlighter, color mode, and background behavior.
///
/// # Errors
///
/// Returns an error if highlighting fails or if rendered spans are invalid.
pub fn highlight_lines_to_ansi_lines_with_highlighter_and_mode_and_background<S: AsRef<str>>(
    highlighter: &mut SpanHighlighter,
    lines: &[S],
    flavor: Grammar,
    theme: &Theme,
    color_mode: ColorMode,
    preserve_terminal_background: bool,
) -> Result<Vec<String>, RenderError> {
    let highlight = highlighter.highlight_lines(lines, flavor)?;
    let source = lines
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>()
        .join("\n");
    let styled = resolve_styled_spans_for_source(source.len(), &highlight, theme)?;
    render_ansi_lines_with_mode_and_background(
        source.as_bytes(),
        &styled,
        color_mode,
        preserve_terminal_background,
    )
}

/// Clips cached styled lines to the current viewport bounds.
fn clip_lines_to_viewport(
    lines: &[Vec<StyledCell>],
    width: usize,
    height: usize,
) -> Vec<Vec<StyledCell>> {
    lines
        .iter()
        .take(height)
        .map(|line| line.iter().take(width).cloned().collect::<Vec<_>>())
        .collect::<Vec<_>>()
}

/// Projects source bytes and spans into styled terminal cells for diffing.
///
/// Cells are grapheme-based and each cell tracks its terminal display width.
fn build_styled_cells(
    source: &[u8],
    spans: &[StyledSpan],
    width: usize,
    height: usize,
) -> Vec<Vec<StyledCell>> {
    let mut lines = Vec::new();
    let mut line = Vec::new();
    let mut line_display_width = 0usize;
    let mut span_cursor = 0usize;

    let rendered = String::from_utf8_lossy(source);
    for (byte_idx, grapheme) in rendered.grapheme_indices(true) {
        while span_cursor < spans.len() && spans[span_cursor].end_byte <= byte_idx {
            span_cursor += 1;
        }

        let style = if let Some(span) = spans.get(span_cursor) {
            if byte_idx >= span.start_byte && byte_idx < span.end_byte {
                span.style
            } else {
                None
            }
        } else {
            None
        };

        if grapheme == "\n" {
            lines.push(line);
            if lines.len() >= height {
                return lines;
            }
            line = Vec::new();
            line_display_width = 0;
            continue;
        }

        let cell_width = display_width_for_grapheme(grapheme, line_display_width);
        if line_display_width + cell_width <= width || cell_width == 0 {
            line.push(StyledCell {
                text: grapheme.to_string(),
                style,
                width: cell_width,
            });
            line_display_width += cell_width;
        }
    }

    lines.push(line);
    lines.truncate(height);
    lines
}

/// Projects a single-line source and spans into styled cells for line diffing.
fn build_styled_line_cells(source: &[u8], spans: &[StyledSpan]) -> Vec<StyledCell> {
    let mut line = Vec::new();
    let mut line_display_width = 0usize;
    let mut span_cursor = 0usize;

    let rendered = String::from_utf8_lossy(source);
    for (byte_idx, grapheme) in rendered.grapheme_indices(true) {
        while span_cursor < spans.len() && spans[span_cursor].end_byte <= byte_idx {
            span_cursor += 1;
        }

        let style = if let Some(span) = spans.get(span_cursor) {
            if byte_idx >= span.start_byte && byte_idx < span.end_byte {
                span.style
            } else {
                None
            }
        } else {
            None
        };

        if grapheme == "\n" {
            break;
        }

        let cell_width = display_width_for_grapheme(grapheme, line_display_width);
        line.push(StyledCell {
            text: grapheme.to_string(),
            style,
            width: cell_width,
        });
        line_display_width = line_display_width.saturating_add(cell_width);
    }

    line
}

/// Returns the terminal display width of a grapheme at a given display column.
fn display_width_for_grapheme(grapheme: &str, line_display_width: usize) -> usize {
    if grapheme == "\t" {
        return tab_width_at(line_display_width, TAB_STOP);
    }
    UnicodeWidthStr::width(grapheme)
}

/// Returns how many display columns a tab advances at `display_col`.
fn tab_width_at(display_col: usize, tab_stop: usize) -> usize {
    let stop = tab_stop.max(1);
    let remainder = display_col % stop;
    if remainder == 0 {
        stop
    } else {
        stop - remainder
    }
}

/// Builds a VT patch by diffing previous and current styled lines.
///
/// `origin_row` and `origin_col` are 1-based terminal coordinates.
/// Column calculations are display-width based (not byte-count based).
fn diff_lines_to_patch(
    prev_lines: &[Vec<StyledCell>],
    curr_lines: &[Vec<StyledCell>],
    origin_row: usize,
    origin_col: usize,
    color_mode: ColorMode,
    preserve_terminal_background: bool,
) -> String {
    let mut out = String::new();
    let line_count = prev_lines.len().max(curr_lines.len());
    let origin_row0 = origin_row.saturating_sub(1);
    let origin_col0 = origin_col.saturating_sub(1);

    for row in 0..line_count {
        let prev = prev_lines.get(row).map(Vec::as_slice).unwrap_or(&[]);
        let curr = curr_lines.get(row).map(Vec::as_slice).unwrap_or(&[]);

        let Some(first_diff) = first_diff_index(prev, curr) else {
            continue;
        };

        let diff_col = display_columns_before(curr, first_diff) + 1;
        let absolute_row = origin_row0 + row + 1;
        let absolute_col = origin_col0 + diff_col;
        write_cup(&mut out, absolute_row, absolute_col);
        append_styled_cells(
            &mut out,
            &curr[first_diff..],
            color_mode,
            preserve_terminal_background,
        );

        if curr.len() < prev.len() {
            out.push_str(EL_TO_END);
        }
    }

    out
}

/// Builds a single-line VT patch using only relative backward cursor motion.
///
/// Assumes cursor starts at the end of `prev_line` and remains on the same line.
fn diff_single_line_to_patch(
    prev_line: &[StyledCell],
    curr_line: &[StyledCell],
    color_mode: ColorMode,
    preserve_terminal_background: bool,
) -> String {
    let mut out = String::new();
    let Some(first_diff) = first_diff_index(prev_line, curr_line) else {
        return out;
    };

    let prev_width = display_columns_before(prev_line, prev_line.len());
    let prefix_width = display_columns_before(prev_line, first_diff);
    let cols_back = prev_width.saturating_sub(prefix_width);
    write_cub(&mut out, cols_back);
    append_styled_cells(
        &mut out,
        &curr_line[first_diff..],
        color_mode,
        preserve_terminal_background,
    );
    if curr_line.len() < prev_line.len() {
        out.push_str(EL_TO_END);
    }
    out
}

/// Returns the accumulated display columns before `idx`.
fn display_columns_before(cells: &[StyledCell], idx: usize) -> usize {
    cells.iter().take(idx).map(|cell| cell.width).sum::<usize>()
}

/// Returns the first differing cell index between two lines, if any.
fn first_diff_index(prev: &[StyledCell], curr: &[StyledCell]) -> Option<usize> {
    let shared = prev.len().min(curr.len());
    for idx in 0..shared {
        if prev[idx] != curr[idx] {
            return Some(idx);
        }
    }

    if prev.len() != curr.len() {
        return Some(shared);
    }

    None
}

/// Writes a CUP cursor-position sequence into `out`.
fn write_cup(out: &mut String, row: usize, col: usize) {
    let _ = write!(out, "{CSI}{row};{col}H");
}

/// Writes a relative cursor-left (CUB) sequence into `out`.
fn write_cub(out: &mut String, cols: usize) {
    if cols == 0 {
        return;
    }
    let _ = write!(out, "{CSI}{cols}D");
}

/// Appends styled cells as text and SGR transitions.
fn append_styled_cells(
    out: &mut String,
    cells: &[StyledCell],
    color_mode: ColorMode,
    preserve_terminal_background: bool,
) {
    if cells.is_empty() {
        return;
    }

    let mut active_style = None;
    for cell in cells {
        write_style_transition(
            out,
            active_style,
            cell.style,
            color_mode,
            preserve_terminal_background,
        );
        out.push_str(&cell.text);
        active_style = cell.style;
    }

    if active_style.is_some() {
        out.push_str(SGR_RESET);
    }
}

/// Emits the minimal SGR transition between `previous` and `next`.
fn write_style_transition(
    out: &mut String,
    previous: Option<Style>,
    next: Option<Style>,
    color_mode: ColorMode,
    preserve_terminal_background: bool,
) {
    if previous == next {
        return;
    }

    match (previous, next) {
        (None, None) => {}
        (Some(_), None) => out.push_str(SGR_RESET),
        (None, Some(style)) => {
            if let Some(open) =
                style_open_sgr(Some(style), color_mode, preserve_terminal_background)
            {
                out.push_str(&open);
            }
        }
        (Some(_), Some(style)) => {
            out.push_str(SGR_RESET);
            if let Some(open) =
                style_open_sgr(Some(style), color_mode, preserve_terminal_background)
            {
                out.push_str(&open);
            }
        }
    }
}

/// Appends a single styled byte segment to `out`.
fn append_styled_segment(
    out: &mut String,
    text: &[u8],
    style: Option<Style>,
    color_mode: ColorMode,
    preserve_terminal_background: bool,
) {
    if text.is_empty() {
        return;
    }

    if let Some(open) = style_open_sgr(style, color_mode, preserve_terminal_background) {
        out.push_str(&open);
        out.push_str(&String::from_utf8_lossy(text));
        out.push_str(SGR_RESET);
        return;
    }

    out.push_str(&String::from_utf8_lossy(text));
}

/// Converts a style into an opening SGR sequence.
///
/// Returns `None` when the style carries no terminal attributes.
fn style_open_sgr(
    style: Option<Style>,
    color_mode: ColorMode,
    preserve_terminal_background: bool,
) -> Option<String> {
    let style = style?;
    let mut parts = Vec::new();
    if let Some(fg) = style.fg {
        let sgr = match color_mode {
            ColorMode::TrueColor => format!("38;2;{};{};{}", fg.r, fg.g, fg.b),
            ColorMode::Ansi256 => format!("38;5;{}", rgb_to_ansi256(fg.r, fg.g, fg.b)),
            ColorMode::Ansi16 => format!("{}", ansi16_fg_sgr(rgb_to_ansi16(fg.r, fg.g, fg.b))),
        };
        parts.push(sgr);
    }
    if !preserve_terminal_background {
        if let Some(bg) = style.bg {
            let sgr = match color_mode {
                ColorMode::TrueColor => format!("48;2;{};{};{}", bg.r, bg.g, bg.b),
                ColorMode::Ansi256 => format!("48;5;{}", rgb_to_ansi256(bg.r, bg.g, bg.b)),
                ColorMode::Ansi16 => {
                    format!("{}", ansi16_bg_sgr(rgb_to_ansi16(bg.r, bg.g, bg.b)))
                }
            };
            parts.push(sgr);
        }
    }
    if style.bold {
        parts.push("1".to_string());
    }
    if style.italic {
        parts.push("3".to_string());
    }
    if style.underline {
        parts.push("4".to_string());
    }

    if parts.is_empty() {
        return None;
    }

    Some(format!("\x1b[{}m", parts.join(";")))
}

/// Quantizes an RGB color to the nearest ANSI 256-color palette index.
fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    let (r_idx, r_level) = nearest_ansi_level(r);
    let (g_idx, g_level) = nearest_ansi_level(g);
    let (b_idx, b_level) = nearest_ansi_level(b);

    let cube_index = 16 + (36 * r_idx) + (6 * g_idx) + b_idx;
    let cube_distance = squared_distance((r, g, b), (r_level, g_level, b_level));

    let gray_index = (((i32::from(r) + i32::from(g) + i32::from(b)) / 3 - 8 + 5) / 10).clamp(0, 23);
    let gray_level = (8 + gray_index * 10) as u8;
    let gray_distance = squared_distance((r, g, b), (gray_level, gray_level, gray_level));

    if gray_distance < cube_distance {
        232 + gray_index as u8
    } else {
        cube_index as u8
    }
}

/// Quantizes an RGB color to an ANSI 16-color palette index.
///
/// This mapping favors hue preservation over pure Euclidean distance so
/// pastel/editor-theme colors do not collapse into mostly white/gray.
fn rgb_to_ansi16(r: u8, g: u8, b: u8) -> usize {
    let rf = f32::from(r) / 255.0;
    let gf = f32::from(g) / 255.0;
    let bf = f32::from(b) / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;

    // Low-saturation colors map to grayscale variants.
    if delta < 0.08 || (max > 0.0 && (delta / max) < 0.18) {
        return if max < 0.20 {
            0
        } else if max < 0.45 {
            8
        } else if max < 0.80 {
            7
        } else {
            15
        };
    }

    let mut hue = if (max - rf).abs() < f32::EPSILON {
        60.0 * ((gf - bf) / delta).rem_euclid(6.0)
    } else if (max - gf).abs() < f32::EPSILON {
        60.0 * (((bf - rf) / delta) + 2.0)
    } else {
        60.0 * (((rf - gf) / delta) + 4.0)
    };
    if hue < 0.0 {
        hue += 360.0;
    }

    let base = if !(30.0..330.0).contains(&hue) {
        1 // red
    } else if hue < 90.0 {
        3 // yellow
    } else if hue < 150.0 {
        2 // green
    } else if hue < 210.0 {
        6 // cyan
    } else if hue < 270.0 {
        4 // blue
    } else {
        5 // magenta
    };

    // Bright variant for lighter colors.
    let bright = max >= 0.62;
    if bright {
        base + 8
    } else {
        base
    }
}

/// Returns the ANSI SGR foreground code for a 16-color palette index.
fn ansi16_fg_sgr(index: usize) -> u8 {
    if index < 8 {
        30 + index as u8
    } else {
        90 + (index as u8 - 8)
    }
}

/// Returns the ANSI SGR background code for a 16-color palette index.
fn ansi16_bg_sgr(index: usize) -> u8 {
    if index < 8 {
        40 + index as u8
    } else {
        100 + (index as u8 - 8)
    }
}

/// Returns the nearest ANSI cube level index and channel value.
fn nearest_ansi_level(value: u8) -> (usize, u8) {
    let mut best_idx = 0usize;
    let mut best_diff = i16::MAX;
    for (idx, level) in ANSI_256_LEVELS.iter().enumerate() {
        let diff = (i16::from(value) - i16::from(*level)).abs();
        if diff < best_diff {
            best_diff = diff;
            best_idx = idx;
        }
    }
    (best_idx, ANSI_256_LEVELS[best_idx])
}

/// Returns squared Euclidean distance in RGB space.
fn squared_distance(lhs: (u8, u8, u8), rhs: (u8, u8, u8)) -> i32 {
    let dr = i32::from(lhs.0) - i32::from(rhs.0);
    let dg = i32::from(lhs.1) - i32::from(rhs.1);
    let db = i32::from(lhs.2) - i32::from(rhs.2);
    (dr * dr) + (dg * dg) + (db * db)
}

/// Returns byte ranges for each line in `source` (excluding trailing newlines).
fn compute_line_ranges(source: &[u8]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut line_start = 0usize;
    for (i, byte) in source.iter().enumerate() {
        if *byte == b'\n' {
            ranges.push((line_start, i));
            line_start = i + 1;
        }
    }
    ranges.push((line_start, source.len()));
    ranges
}

/// Validates that spans are in-bounds, sorted, and non-overlapping.
///
/// # Errors
///
/// Returns [`RenderError::SpanOutOfBounds`] or [`RenderError::OverlappingSpans`]
/// when invariants are violated.
fn validate_spans(source_len: usize, spans: &[StyledSpan]) -> Result<(), RenderError> {
    let mut prev_end = 0usize;
    for (i, span) in spans.iter().enumerate() {
        if span.start_byte > span.end_byte || span.end_byte > source_len {
            return Err(RenderError::SpanOutOfBounds {
                start_byte: span.start_byte,
                end_byte: span.end_byte,
                source_len,
            });
        }
        if i > 0 && span.start_byte < prev_end {
            return Err(RenderError::OverlappingSpans {
                prev_end,
                next_start: span.start_byte,
            });
        }
        prev_end = span.end_byte;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        highlight_lines_to_ansi_lines, highlight_to_ansi, render_ansi, render_ansi_lines,
        render_ansi_with_mode, render_ansi_with_mode_and_background, resolve_styled_spans,
        resolve_styled_spans_for_source, ColorMode, IncrementalRenderer, RenderError,
        StreamLineRenderer, StyledSpan,
    };
    use highlight_spans::{Attr, Grammar, HighlightResult, Span, SpanHighlighter};
    use theme_engine::{load_theme, Rgb, Style, Theme};

    #[test]
    /// Verifies a styled segment is wrapped with expected SGR codes.
    fn renders_basic_styled_segment() {
        let source = b"abc";
        let spans = [StyledSpan {
            start_byte: 1,
            end_byte: 2,
            style: Some(Style {
                fg: Some(Rgb::new(255, 0, 0)),
                bold: true,
                ..Style::default()
            }),
        }];
        let out = render_ansi(source, &spans).expect("failed to render");
        assert_eq!(out, "a\x1b[38;2;255;0;0;1mb\x1b[0mc");
    }

    #[test]
    /// Verifies OSC helpers emit expected set/reset default color sequences.
    fn emits_osc_default_color_sequences() {
        let fg = Rgb::new(1, 2, 3);
        let bg = Rgb::new(4, 5, 6);
        assert_eq!(super::osc_set_default_foreground(fg), "\x1b]10;#010203\x07");
        assert_eq!(super::osc_set_default_background(bg), "\x1b]11;#040506\x07");
        assert_eq!(
            super::osc_set_default_colors(Some(fg), Some(bg)),
            "\x1b]10;#010203\x07\x1b]11;#040506\x07"
        );
        assert_eq!(super::osc_reset_default_foreground(), "\x1b]110\x07");
        assert_eq!(super::osc_reset_default_background(), "\x1b]111\x07");
        assert_eq!(
            super::osc_reset_default_colors(),
            "\x1b]110\x07\x1b]111\x07"
        );
    }

    #[test]
    /// Verifies OSC theme defaults fall back to `normal` style colors.
    fn emits_osc_default_colors_from_theme() {
        let theme = Theme::from_json_str(
            r#"{
  "styles": {
    "normal": { "fg": { "r": 10, "g": 11, "b": 12 }, "bg": { "r": 13, "g": 14, "b": 15 } }
  }
}"#,
        )
        .expect("theme parse failed");
        let osc = super::osc_set_default_colors_from_theme(&theme);
        assert_eq!(osc, "\x1b]10;#0a0b0c\x07\x1b]11;#0d0e0f\x07");
    }

    #[test]
    /// Verifies stream line renderer paints the initial line as-is.
    fn stream_line_renderer_emits_initial_line() {
        let mut renderer = StreamLineRenderer::new();
        let patch = renderer
            .render_line_patch(b"hello", &[])
            .expect("initial stream patch failed");
        assert_eq!(patch, "hello");
    }

    #[test]
    /// Verifies stream line renderer emits a suffix-only patch with CUB backtracking.
    fn stream_line_renderer_emits_suffix_with_backtracking() {
        let mut renderer = StreamLineRenderer::new();
        let _ = renderer
            .render_line_patch(b"hello", &[])
            .expect("initial stream patch failed");
        let patch = renderer
            .render_line_patch(b"heLlo", &[])
            .expect("delta stream patch failed");
        assert_eq!(patch, "\x1b[3DLlo");
    }

    #[test]
    /// Verifies stream line renderer clears trailing cells when line becomes shorter.
    fn stream_line_renderer_clears_removed_tail() {
        let mut renderer = StreamLineRenderer::new();
        let _ = renderer
            .render_line_patch(b"hello", &[])
            .expect("initial stream patch failed");
        let patch = renderer
            .render_line_patch(b"he", &[])
            .expect("delta stream patch failed");
        assert_eq!(patch, "\x1b[3D\x1b[K");
    }

    #[test]
    /// Verifies stream line renderer emits nothing for unchanged input.
    fn stream_line_renderer_is_noop_when_unchanged() {
        let mut renderer = StreamLineRenderer::new();
        let _ = renderer
            .render_line_patch(b"hello", &[])
            .expect("initial stream patch failed");
        let patch = renderer
            .render_line_patch(b"hello", &[])
            .expect("delta stream patch failed");
        assert!(patch.is_empty());
    }

    #[test]
    /// Verifies stream line renderer rejects multi-line input.
    fn stream_line_renderer_rejects_multiline_input() {
        let mut renderer = StreamLineRenderer::new();
        let err = renderer
            .render_line_patch(b"hello\nworld", &[])
            .expect_err("expected multiline rejection");
        assert!(matches!(err, RenderError::MultiLineInput));
    }

    #[test]
    /// Verifies stream line backtracking uses display width for wide graphemes.
    fn stream_line_renderer_uses_display_width_for_wide_graphemes() {
        let mut renderer = StreamLineRenderer::new();
        let _ = renderer
            .render_line_patch("a界!".as_bytes(), &[])
            .expect("initial stream patch failed");
        let patch = renderer
            .render_line_patch("a界?".as_bytes(), &[])
            .expect("delta stream patch failed");
        assert_eq!(patch, "\x1b[1D?");
    }

    #[test]
    /// Verifies ANSI-256 mode emits indexed foreground color SGR.
    fn renders_ansi256_styled_segment() {
        let source = b"abc";
        let spans = [StyledSpan {
            start_byte: 1,
            end_byte: 2,
            style: Some(Style {
                fg: Some(Rgb::new(255, 0, 0)),
                bold: true,
                ..Style::default()
            }),
        }];
        let out =
            render_ansi_with_mode(source, &spans, ColorMode::Ansi256).expect("failed to render");
        assert_eq!(out, "a\x1b[38;5;196;1mb\x1b[0mc");
    }

    #[test]
    /// Verifies ANSI-16 mode emits basic/bright indexed foreground color SGR.
    fn renders_ansi16_styled_segment() {
        let source = b"abc";
        let spans = [StyledSpan {
            start_byte: 1,
            end_byte: 2,
            style: Some(Style {
                fg: Some(Rgb::new(255, 0, 0)),
                bold: true,
                ..Style::default()
            }),
        }];
        let out =
            render_ansi_with_mode(source, &spans, ColorMode::Ansi16).expect("failed to render");
        assert_eq!(out, "a\x1b[91;1mb\x1b[0mc");
    }

    #[test]
    /// Verifies ANSI-16 mode keeps hue for saturated non-red colors.
    fn renders_ansi16_preserves_non_gray_hue() {
        let source = b"abc";
        let spans = [StyledSpan {
            start_byte: 1,
            end_byte: 2,
            style: Some(Style {
                fg: Some(Rgb::new(130, 170, 255)),
                ..Style::default()
            }),
        }];
        let out =
            render_ansi_with_mode(source, &spans, ColorMode::Ansi16).expect("failed to render");
        assert!(
            out.contains("\x1b[94m"),
            "expected bright blue ANSI16 code, got {out:?}"
        );
    }

    #[test]
    /// Verifies theme background colors are emitted when passthrough is disabled.
    fn renders_theme_background_when_passthrough_disabled() {
        let source = b"a";
        let spans = [StyledSpan {
            start_byte: 0,
            end_byte: 1,
            style: Some(Style {
                fg: Some(Rgb::new(255, 0, 0)),
                bg: Some(Rgb::new(1, 2, 3)),
                ..Style::default()
            }),
        }];
        let out = render_ansi_with_mode_and_background(source, &spans, ColorMode::TrueColor, false)
            .expect("failed to render");
        assert_eq!(out, "\x1b[38;2;255;0;0;48;2;1;2;3ma\x1b[0m");
    }

    #[test]
    /// Verifies stream line renderer can emit themed background colors.
    fn stream_line_renderer_emits_theme_background_when_enabled() {
        let mut renderer = StreamLineRenderer::new();
        renderer.set_preserve_terminal_background(false);
        let spans = [StyledSpan {
            start_byte: 0,
            end_byte: 1,
            style: Some(Style {
                fg: Some(Rgb::new(255, 0, 0)),
                bg: Some(Rgb::new(1, 2, 3)),
                ..Style::default()
            }),
        }];
        let patch = renderer
            .render_line_patch(b"a", &spans)
            .expect("initial stream patch failed");
        assert_eq!(patch, "\x1b[38;2;255;0;0;48;2;1;2;3ma\x1b[0m");
    }

    #[test]
    /// Verifies multiline spans are clipped and rendered per line.
    fn renders_per_line_output_for_multiline_span() {
        let source = b"ab\ncd";
        let spans = [StyledSpan {
            start_byte: 1,
            end_byte: 5,
            style: Some(Style {
                fg: Some(Rgb::new(1, 2, 3)),
                ..Style::default()
            }),
        }];

        let lines = render_ansi_lines(source, &spans).expect("failed to render lines");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "a\x1b[38;2;1;2;3mb\x1b[0m");
        assert_eq!(lines[1], "\x1b[38;2;1;2;3mcd\x1b[0m");
    }

    #[test]
    /// Verifies overlapping spans are rejected.
    fn rejects_overlapping_spans() {
        let spans = [
            StyledSpan {
                start_byte: 0,
                end_byte: 2,
                style: None,
            },
            StyledSpan {
                start_byte: 1,
                end_byte: 3,
                style: None,
            },
        ];
        let err = render_ansi(b"abcd", &spans).expect_err("expected overlap error");
        assert!(matches!(err, RenderError::OverlappingSpans { .. }));
    }

    #[test]
    /// Verifies end-to-end highlight plus ANSI rendering works.
    fn highlights_source_to_ansi() {
        let theme = Theme::from_json_str(
            r#"{
  "styles": {
    "normal": { "fg": { "r": 220, "g": 220, "b": 220 } },
    "number": { "fg": { "r": 255, "g": 180, "b": 120 } }
  }
}"#,
        )
        .expect("theme parse failed");

        let out = highlight_to_ansi(b"set x = 42", Grammar::ObjectScript, &theme)
            .expect("highlight+render failed");
        assert!(out.contains("42"));
        assert!(out.contains("\x1b["));
    }

    #[test]
    /// Verifies capture styles inherit missing fields from `normal`.
    fn resolve_styled_spans_inherits_from_normal() {
        let theme = Theme::from_json_str(
            r#"{
  "styles": {
    "normal": {
      "fg": { "r": 10, "g": 11, "b": 12 },
      "bg": { "r": 200, "g": 201, "b": 202 },
      "italic": true
    },
    "keyword": { "fg": { "r": 250, "g": 1, "b": 2 } }
  }
}"#,
        )
        .expect("theme parse failed");
        let highlight = HighlightResult {
            attrs: vec![Attr {
                id: 0,
                capture_name: "keyword".to_string(),
            }],
            spans: vec![Span {
                attr_id: 0,
                start_byte: 0,
                end_byte: 3,
            }],
        };

        let styled = resolve_styled_spans(&highlight, &theme).expect("resolve failed");
        assert_eq!(styled.len(), 1);
        let style = styled[0].style.expect("missing style");
        assert_eq!(style.fg, Some(Rgb::new(250, 1, 2)));
        assert_eq!(style.bg, Some(Rgb::new(200, 201, 202)));
        assert!(style.italic);
    }

    #[test]
    /// Verifies high-level resolution fills uncovered byte ranges with `normal`.
    fn resolve_styled_spans_for_source_fills_uncovered_ranges() {
        let theme = Theme::from_json_str(
            r#"{
  "styles": {
    "normal": {
      "fg": { "r": 1, "g": 2, "b": 3 },
      "bg": { "r": 4, "g": 5, "b": 6 }
    },
    "keyword": { "fg": { "r": 7, "g": 8, "b": 9 } }
  }
}"#,
        )
        .expect("theme parse failed");
        let highlight = HighlightResult {
            attrs: vec![Attr {
                id: 0,
                capture_name: "keyword".to_string(),
            }],
            spans: vec![Span {
                attr_id: 0,
                start_byte: 1,
                end_byte: 2,
            }],
        };

        let styled =
            resolve_styled_spans_for_source(4, &highlight, &theme).expect("resolve failed");
        assert_eq!(styled.len(), 3);
        assert_eq!(
            styled[0],
            StyledSpan {
                start_byte: 0,
                end_byte: 1,
                style: Some(Style {
                    fg: Some(Rgb::new(1, 2, 3)),
                    bg: Some(Rgb::new(4, 5, 6)),
                    ..Style::default()
                }),
            }
        );
        assert_eq!(
            styled[1],
            StyledSpan {
                start_byte: 1,
                end_byte: 2,
                style: Some(Style {
                    fg: Some(Rgb::new(7, 8, 9)),
                    bg: Some(Rgb::new(4, 5, 6)),
                    ..Style::default()
                }),
            }
        );
        assert_eq!(
            styled[2],
            StyledSpan {
                start_byte: 2,
                end_byte: 4,
                style: Some(Style {
                    fg: Some(Rgb::new(1, 2, 3)),
                    bg: Some(Rgb::new(4, 5, 6)),
                    ..Style::default()
                }),
            }
        );
    }

    #[test]
    /// Verifies line-oriented highlight rendering preserves line count.
    fn highlights_lines_to_ansi_lines() {
        let theme = load_theme("tokyo-night").expect("failed to load built-in theme");
        let lines = vec!["set x = 1", "set y = 2"];
        let rendered = highlight_lines_to_ansi_lines(&lines, Grammar::ObjectScript, &theme)
            .expect("failed to highlight lines");
        assert_eq!(rendered.len(), 2);
    }

    #[test]
    /// Verifies incremental patches include only changed line suffixes.
    fn incremental_renderer_emits_only_changed_line_suffix() {
        let mut renderer = IncrementalRenderer::new(120, 40);
        let first = renderer
            .render_patch(b"abc\nxyz", &[])
            .expect("first patch failed");
        assert!(first.contains("\x1b[1;1Habc"));
        assert!(first.contains("\x1b[2;1Hxyz"));

        let second = renderer
            .render_patch(b"abc\nxYz", &[])
            .expect("second patch failed");
        assert_eq!(second, "\x1b[2;2HYz");

        let third = renderer
            .render_patch(b"abc\nxYz", &[])
            .expect("third patch failed");
        assert!(third.is_empty());
    }

    #[test]
    /// Verifies configured origin offsets CUP coordinates in emitted patches.
    fn incremental_renderer_applies_origin_offset() {
        let mut renderer = IncrementalRenderer::new(120, 40);
        renderer.set_origin(4, 7);

        let first = renderer
            .render_patch(b"abc", &[])
            .expect("first patch failed");
        assert_eq!(first, "\x1b[4;7Habc");

        let second = renderer
            .render_patch(b"abC", &[])
            .expect("second patch failed");
        assert_eq!(second, "\x1b[4;9HC");
    }

    #[test]
    /// Verifies incremental renderer can emit ANSI-256 foreground colors.
    fn incremental_renderer_supports_ansi256_mode() {
        let mut renderer = IncrementalRenderer::new(120, 40);
        renderer.set_color_mode(ColorMode::Ansi256);
        let spans = [StyledSpan {
            start_byte: 0,
            end_byte: 2,
            style: Some(Style {
                fg: Some(Rgb::new(255, 0, 0)),
                ..Style::default()
            }),
        }];

        let patch = renderer
            .render_patch(b"ab", &spans)
            .expect("patch generation failed");
        assert!(patch.contains("\x1b[38;5;196m"));
        assert!(!patch.contains("38;2;"));
    }

    #[test]
    /// Verifies incremental renderer can emit ANSI-16 foreground colors.
    fn incremental_renderer_supports_ansi16_mode() {
        let mut renderer = IncrementalRenderer::new(120, 40);
        renderer.set_color_mode(ColorMode::Ansi16);
        let spans = [StyledSpan {
            start_byte: 0,
            end_byte: 2,
            style: Some(Style {
                fg: Some(Rgb::new(255, 0, 0)),
                ..Style::default()
            }),
        }];

        let patch = renderer
            .render_patch(b"ab", &spans)
            .expect("patch generation failed");
        assert!(patch.contains("\x1b[91m"));
        assert!(!patch.contains("38;2;"));
        assert!(!patch.contains("38;5;"));
    }

    #[test]
    /// Verifies incremental renderer can emit themed background colors.
    fn incremental_renderer_emits_theme_background_when_enabled() {
        let mut renderer = IncrementalRenderer::new(120, 40);
        renderer.set_preserve_terminal_background(false);
        let spans = [StyledSpan {
            start_byte: 0,
            end_byte: 1,
            style: Some(Style {
                fg: Some(Rgb::new(255, 0, 0)),
                bg: Some(Rgb::new(1, 2, 3)),
                ..Style::default()
            }),
        }];

        let patch = renderer
            .render_patch(b"a", &spans)
            .expect("patch generation failed");
        assert!(patch.contains("\x1b[38;2;255;0;0;48;2;1;2;3ma"));
    }

    #[test]
    /// Verifies CUP columns account for wide grapheme display widths.
    fn incremental_renderer_uses_display_width_for_wide_graphemes() {
        let mut renderer = IncrementalRenderer::new(120, 40);
        let _ = renderer
            .render_patch("a界".as_bytes(), &[])
            .expect("first patch failed");

        let patch = renderer
            .render_patch("a界!".as_bytes(), &[])
            .expect("second patch failed");
        assert_eq!(patch, "\x1b[1;4H!");
    }

    #[test]
    /// Verifies tab cells advance to the next configured tab stop for CUP columns.
    fn incremental_renderer_uses_display_width_for_tabs() {
        let mut renderer = IncrementalRenderer::new(120, 40);
        let _ = renderer
            .render_patch(b"a\tb", &[])
            .expect("first patch failed");

        let patch = renderer
            .render_patch(b"a\tB", &[])
            .expect("second patch failed");
        assert_eq!(patch, "\x1b[1;9HB");
    }

    #[test]
    /// Verifies incremental patches clear removed trailing cells.
    fn incremental_renderer_clears_removed_tail() {
        let mut renderer = IncrementalRenderer::new(120, 40);
        let _ = renderer
            .render_patch(b"hello", &[])
            .expect("first patch failed");

        let patch = renderer
            .render_patch(b"he", &[])
            .expect("second patch failed");
        assert_eq!(patch, "\x1b[1;3H\x1b[K");
    }

    #[test]
    /// Verifies incremental rendering works with the highlight pipeline.
    fn incremental_renderer_supports_highlight_pipeline() {
        let theme = Theme::from_json_str(
            r#"{
  "styles": {
    "normal": { "fg": { "r": 220, "g": 220, "b": 220 } },
    "keyword": { "fg": { "r": 255, "g": 0, "b": 0 } }
  }
}"#,
        )
        .expect("theme parse failed");
        let mut highlighter = SpanHighlighter::new().expect("highlighter init failed");
        let mut renderer = IncrementalRenderer::new(120, 40);

        let patch = renderer
            .highlight_to_patch(&mut highlighter, b"SELECT 1", Grammar::Sql, &theme)
            .expect("highlight patch failed");
        assert!(patch.contains("\x1b[1;1H"));
        assert!(patch.contains("SELECT"));
    }
}
