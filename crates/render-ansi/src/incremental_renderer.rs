use crate::ansi_structures::{ColorMode, IncrementalRenderer, RenderError, StyledSpan};
use crate::common::{
    build_styled_cells, clip_lines_to_viewport, diff_lines_to_patch,
    resolve_styled_spans_for_source, validate_spans,
};
use highlight_spans::highlight_structures::{Grammar, SpanHighlighter};
use theme_engine::theme_structures::Theme;

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
            preserve_terminal_background: false,
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
