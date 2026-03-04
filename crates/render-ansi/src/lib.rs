use std::collections::HashMap;
use std::fmt::Write;

use highlight_spans::{Grammar, HighlightError, HighlightResult, SpanHighlighter};
use theme_engine::{Style, Theme};
use thiserror::Error;

const CSI: &str = "\x1b[";
const SGR_RESET: &str = "\x1b[0m";
const EL_TO_END: &str = "\x1b[K";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StyledSpan {
    pub start_byte: usize,
    pub end_byte: usize,
    pub style: Option<Style>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct StyledCell {
    ch: char,
    style: Option<Style>,
}

#[derive(Debug, Clone)]
pub struct IncrementalRenderer {
    width: usize,
    height: usize,
    prev_lines: Vec<Vec<StyledCell>>,
}

impl IncrementalRenderer {
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width: width.max(1),
            height: height.max(1),
            prev_lines: Vec::new(),
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.prev_lines = clip_lines_to_viewport(&self.prev_lines, self.width, self.height);
    }

    pub fn clear_state(&mut self) {
        self.prev_lines.clear();
    }

    pub fn render_patch(
        &mut self,
        source: &[u8],
        spans: &[StyledSpan],
    ) -> Result<String, RenderError> {
        validate_spans(source.len(), spans)?;
        let curr_lines = build_styled_cells(source, spans, self.width, self.height);
        let patch = diff_lines_to_patch(&self.prev_lines, &curr_lines);
        self.prev_lines = curr_lines;
        Ok(patch)
    }

    pub fn highlight_to_patch(
        &mut self,
        highlighter: &mut SpanHighlighter,
        source: &[u8],
        flavor: Grammar,
        theme: &Theme,
    ) -> Result<String, RenderError> {
        let highlight = highlighter.highlight(source, flavor)?;
        let styled = resolve_styled_spans(&highlight, theme)?;
        self.render_patch(source, &styled)
    }
}

#[derive(Debug, Clone)]
pub struct IncrementalSessionManager {
    default_width: usize,
    default_height: usize,
    sessions: HashMap<String, IncrementalRenderer>,
}

impl IncrementalSessionManager {
    #[must_use]
    pub fn new(default_width: usize, default_height: usize) -> Self {
        Self {
            default_width: default_width.max(1),
            default_height: default_height.max(1),
            sessions: HashMap::new(),
        }
    }

    #[must_use]
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn ensure_session(&mut self, session_id: &str) -> &mut IncrementalRenderer {
        self.sessions
            .entry(session_id.to_string())
            .or_insert_with(|| IncrementalRenderer::new(self.default_width, self.default_height))
    }

    pub fn ensure_session_with_size(
        &mut self,
        session_id: &str,
        width: usize,
        height: usize,
    ) -> &mut IncrementalRenderer {
        let renderer = self.ensure_session(session_id);
        renderer.resize(width, height);
        renderer
    }

    pub fn remove_session(&mut self, session_id: &str) -> bool {
        self.sessions.remove(session_id).is_some()
    }

    pub fn clear_session(&mut self, session_id: &str) -> bool {
        let Some(renderer) = self.sessions.get_mut(session_id) else {
            return false;
        };
        renderer.clear_state();
        true
    }

    pub fn render_patch_for_session(
        &mut self,
        session_id: &str,
        source: &[u8],
        spans: &[StyledSpan],
    ) -> Result<String, RenderError> {
        self.ensure_session(session_id).render_patch(source, spans)
    }

    pub fn highlight_to_patch_for_session(
        &mut self,
        session_id: &str,
        highlighter: &mut SpanHighlighter,
        source: &[u8],
        flavor: Grammar,
        theme: &Theme,
    ) -> Result<String, RenderError> {
        self.ensure_session(session_id)
            .highlight_to_patch(highlighter, source, flavor, theme)
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
}

pub fn resolve_styled_spans(
    highlight: &HighlightResult,
    theme: &Theme,
) -> Result<Vec<StyledSpan>, RenderError> {
    let mut out = Vec::with_capacity(highlight.spans.len());
    for span in &highlight.spans {
        let Some(attr) = highlight.attrs.get(span.attr_id) else {
            return Err(RenderError::InvalidAttrId {
                attr_id: span.attr_id,
                attrs_len: highlight.attrs.len(),
            });
        };
        out.push(StyledSpan {
            start_byte: span.start_byte,
            end_byte: span.end_byte,
            style: theme.resolve(&attr.capture_name).copied(),
        });
    }
    Ok(out)
}

pub fn render_ansi(source: &[u8], spans: &[StyledSpan]) -> Result<String, RenderError> {
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
        );
        cursor = span.end_byte;
    }

    if cursor < source.len() {
        out.push_str(&String::from_utf8_lossy(&source[cursor..]));
    }

    Ok(out)
}

pub fn render_ansi_lines(source: &[u8], spans: &[StyledSpan]) -> Result<Vec<String>, RenderError> {
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
            append_styled_segment(&mut line, &source[seg_start..seg_end], span.style);
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

pub fn highlight_to_ansi(
    source: &[u8],
    flavor: Grammar,
    theme: &Theme,
) -> Result<String, RenderError> {
    let mut highlighter = SpanHighlighter::new()?;
    highlight_to_ansi_with_highlighter(&mut highlighter, source, flavor, theme)
}

pub fn highlight_to_ansi_with_highlighter(
    highlighter: &mut SpanHighlighter,
    source: &[u8],
    flavor: Grammar,
    theme: &Theme,
) -> Result<String, RenderError> {
    let highlight = highlighter.highlight(source, flavor)?;
    let styled = resolve_styled_spans(&highlight, theme)?;
    render_ansi(source, &styled)
}

pub fn highlight_lines_to_ansi_lines<S: AsRef<str>>(
    lines: &[S],
    flavor: Grammar,
    theme: &Theme,
) -> Result<Vec<String>, RenderError> {
    let mut highlighter = SpanHighlighter::new()?;
    highlight_lines_to_ansi_lines_with_highlighter(&mut highlighter, lines, flavor, theme)
}

pub fn highlight_lines_to_ansi_lines_with_highlighter<S: AsRef<str>>(
    highlighter: &mut SpanHighlighter,
    lines: &[S],
    flavor: Grammar,
    theme: &Theme,
) -> Result<Vec<String>, RenderError> {
    let highlight = highlighter.highlight_lines(lines, flavor)?;
    let source = lines
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>()
        .join("\n");
    let styled = resolve_styled_spans(&highlight, theme)?;
    render_ansi_lines(source.as_bytes(), &styled)
}

fn clip_lines_to_viewport(
    lines: &[Vec<StyledCell>],
    width: usize,
    height: usize,
) -> Vec<Vec<StyledCell>> {
    lines
        .iter()
        .take(height)
        .map(|line| line.iter().copied().take(width).collect::<Vec<_>>())
        .collect::<Vec<_>>()
}

fn build_styled_cells(
    source: &[u8],
    spans: &[StyledSpan],
    width: usize,
    height: usize,
) -> Vec<Vec<StyledCell>> {
    let mut lines = Vec::new();
    let mut line = Vec::new();
    let mut span_cursor = 0usize;

    for (byte_idx, ch) in String::from_utf8_lossy(source).char_indices() {
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

        if ch == '\n' {
            lines.push(line);
            if lines.len() >= height {
                return lines;
            }
            line = Vec::new();
            continue;
        }

        if line.len() < width {
            line.push(StyledCell { ch, style });
        }
    }

    lines.push(line);
    lines.truncate(height);
    lines
}

fn diff_lines_to_patch(prev_lines: &[Vec<StyledCell>], curr_lines: &[Vec<StyledCell>]) -> String {
    let mut out = String::new();
    let line_count = prev_lines.len().max(curr_lines.len());

    for row in 0..line_count {
        let prev = prev_lines.get(row).map(Vec::as_slice).unwrap_or(&[]);
        let curr = curr_lines.get(row).map(Vec::as_slice).unwrap_or(&[]);

        let Some(first_diff) = first_diff_index(prev, curr) else {
            continue;
        };

        write_cup(&mut out, row + 1, first_diff + 1);
        append_styled_cells(&mut out, &curr[first_diff..]);

        if curr.len() < prev.len() {
            out.push_str(EL_TO_END);
        }
    }

    out
}

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

fn write_cup(out: &mut String, row: usize, col: usize) {
    let _ = write!(out, "{CSI}{row};{col}H");
}

fn append_styled_cells(out: &mut String, cells: &[StyledCell]) {
    if cells.is_empty() {
        return;
    }

    let mut active_style = None;
    for cell in cells {
        write_style_transition(out, active_style, cell.style);
        out.push(cell.ch);
        active_style = cell.style;
    }

    if active_style.is_some() {
        out.push_str(SGR_RESET);
    }
}

fn write_style_transition(out: &mut String, previous: Option<Style>, next: Option<Style>) {
    if previous == next {
        return;
    }

    match (previous, next) {
        (None, None) => {}
        (Some(_), None) => out.push_str(SGR_RESET),
        (None, Some(style)) => {
            if let Some(open) = style_open_sgr(Some(style)) {
                out.push_str(&open);
            }
        }
        (Some(_), Some(style)) => {
            out.push_str(SGR_RESET);
            if let Some(open) = style_open_sgr(Some(style)) {
                out.push_str(&open);
            }
        }
    }
}

fn append_styled_segment(out: &mut String, text: &[u8], style: Option<Style>) {
    if text.is_empty() {
        return;
    }

    if let Some(open) = style_open_sgr(style) {
        out.push_str(&open);
        out.push_str(&String::from_utf8_lossy(text));
        out.push_str(SGR_RESET);
        return;
    }

    out.push_str(&String::from_utf8_lossy(text));
}

fn style_open_sgr(style: Option<Style>) -> Option<String> {
    let style = style?;
    let mut parts = Vec::new();
    if let Some(fg) = style.fg {
        parts.push(format!("38;2;{};{};{}", fg.r, fg.g, fg.b));
    }
    if let Some(bg) = style.bg {
        parts.push(format!("48;2;{};{};{}", bg.r, bg.g, bg.b));
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
        IncrementalRenderer, IncrementalSessionManager, RenderError, StyledSpan,
    };
    use highlight_spans::{Grammar, SpanHighlighter};
    use theme_engine::{load_theme, Rgb, Style, Theme};

    #[test]
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
    fn highlights_lines_to_ansi_lines() {
        let theme = load_theme("tokyo-night").expect("failed to load built-in theme");
        let lines = vec!["set x = 1", "set y = 2"];
        let rendered = highlight_lines_to_ansi_lines(&lines, Grammar::ObjectScript, &theme)
            .expect("failed to highlight lines");
        assert_eq!(rendered.len(), 2);
    }

    #[test]
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

    #[test]
    fn session_manager_keeps_incremental_state_per_session() {
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
        let mut manager = IncrementalSessionManager::new(120, 40);

        let a_initial = manager
            .highlight_to_patch_for_session(
                "iris-a",
                &mut highlighter,
                b"SELECT 1",
                Grammar::Sql,
                &theme,
            )
            .expect("a initial patch failed");
        assert!(!a_initial.is_empty());

        let b_initial = manager
            .highlight_to_patch_for_session(
                "iris-b",
                &mut highlighter,
                b"SELECT 1",
                Grammar::Sql,
                &theme,
            )
            .expect("b initial patch failed");
        assert!(!b_initial.is_empty());

        let a_second = manager
            .highlight_to_patch_for_session(
                "iris-a",
                &mut highlighter,
                b"SELECT 2",
                Grammar::Sql,
                &theme,
            )
            .expect("a second patch failed");
        assert!(!a_second.is_empty());

        let b_second = manager
            .highlight_to_patch_for_session(
                "iris-b",
                &mut highlighter,
                b"SELECT 1",
                Grammar::Sql,
                &theme,
            )
            .expect("b second patch failed");
        assert!(
            b_second.is_empty(),
            "session b should have no patch when its own state is unchanged"
        );
    }
}
