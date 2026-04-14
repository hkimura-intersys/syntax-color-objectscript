use crate::ansi_structures::{ColorMode, RenderError, StreamLineRenderer, StyledSpan};
use crate::common::{
    build_styled_line_cells, diff_single_line_to_patch, resolve_styled_spans_for_source,
    validate_spans,
};
use highlight_spans::highlight_structures::{Grammar, SpanHighlighter};
use theme_engine::theme_structures::Theme;

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
            preserve_terminal_background: false,
            prev_line: Vec::new(),
        }
    }
}
