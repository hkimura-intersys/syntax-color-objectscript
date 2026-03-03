use thiserror::Error;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter as TsHighlighter};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Grammar {
    ObjectScript,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Attr {
    pub id: usize,
    pub capture_name: String,
}

impl Attr {
    #[must_use]
    pub fn theme_key(&self) -> String {
        format!("@{}", self.capture_name)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Span {
    pub attr_id: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HighlightResult {
    pub attrs: Vec<Attr>,
    pub spans: Vec<Span>,
}

#[derive(Debug, Error)]
pub enum HighlightError {
    #[error("failed to build highlight configuration: {0}")]
    Query(#[from] tree_sitter::QueryError),
    #[error("highlighting failed: {0}")]
    Highlight(#[from] tree_sitter_highlight::Error),
}

pub struct SpanHighlighter {
    highlighter: TsHighlighter,
    objectscript: HighlightConfiguration,
}

impl SpanHighlighter {
    pub fn new() -> Result<Self, HighlightError> {
        let objectscript = new_config(
            tree_sitter_objectscript::LANGUAGE_OBJECTSCRIPT_PLAYGROUND.into(),
            "objectscript",
            tree_sitter_objectscript::OBJECTSCRIPT_HIGHLIGHTS_QUERY,
            tree_sitter_objectscript::OBJECTSCRIPT_INJECTIONS_QUERY,
        )?;

        Ok(Self {
            highlighter: TsHighlighter::new(),
            objectscript,
        })
    }

    pub fn highlight(
        &mut self,
        source: &[u8],
        flavor: Grammar,
    ) -> Result<HighlightResult, HighlightError> {
        let config = match flavor {
            Grammar::ObjectScript => &self.objectscript,
        };

        let attrs = config
            .names()
            .iter()
            .enumerate()
            .map(|(id, name)| Attr {
                id,
                capture_name: (*name).to_string(),
            })
            .collect::<Vec<_>>();

        let events = self.highlighter.highlight(config, source, None, |_| None)?;
        let mut spans = Vec::new();
        let mut active_stack = Vec::new();

        for event in events {
            match event? {
                HighlightEvent::HighlightStart(highlight) => active_stack.push(highlight.0),
                HighlightEvent::HighlightEnd => {
                    active_stack.pop();
                }
                HighlightEvent::Source { start, end } => {
                    if let Some(&attr_id) = active_stack.last() {
                        push_merged(
                            &mut spans,
                            Span {
                                attr_id,
                                start_byte: start,
                                end_byte: end,
                            },
                        );
                    }
                }
            }
        }

        Ok(HighlightResult { attrs, spans })
    }

    pub fn highlight_lines<S: AsRef<str>>(
        &mut self,
        lines: &[S],
        flavor: Grammar,
    ) -> Result<HighlightResult, HighlightError> {
        let source = lines
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>()
            .join("\n");
        self.highlight(source.as_bytes(), flavor)
    }
}

fn new_config(
    language: tree_sitter::Language,
    language_name: &str,
    highlights: &str,
    injections: &str,
) -> Result<HighlightConfiguration, tree_sitter::QueryError> {
    let mut config =
        HighlightConfiguration::new(language, language_name, highlights, injections, "")?;
    let recognized = config
        .names()
        .iter()
        .map(|name| (*name).to_string())
        .collect::<Vec<_>>();
    let recognized_refs = recognized.iter().map(String::as_str).collect::<Vec<_>>();
    config.configure(&recognized_refs);
    Ok(config)
}

fn push_merged(spans: &mut Vec<Span>, next: Span) {
    if next.start_byte >= next.end_byte {
        return;
    }

    if let Some(last) = spans.last_mut() {
        if last.attr_id == next.attr_id && last.end_byte == next.start_byte {
            last.end_byte = next.end_byte;
            return;
        }
    }

    spans.push(next);
}

#[cfg(test)]
mod tests {
    use super::{Grammar, SpanHighlighter};

    #[test]
    fn highlights_numeric_literal_as_number() {
        let source = br#"
Class Demo.Highlight
{
  ClassMethod Main()
  {
    set x = 42
  }
}
"#;
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::ObjectScript)
            .expect("failed to highlight");

        let number_attr = result
            .attrs
            .iter()
            .find(|attr| attr.capture_name == "number")
            .expect("number capture missing");

        assert!(
            result.spans.iter().any(|span| {
                span.attr_id == number_attr.id && &source[span.start_byte..span.end_byte] == b"42"
            }),
            "expected highlighted span for numeric literal"
        );
    }
}
