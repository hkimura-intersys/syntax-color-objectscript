use std::collections::HashMap;

use thiserror::Error;
use tree_sitter::StreamingIterator;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter as TsHighlighter};

unsafe extern "C" {
    /// Returns the SQL Tree-sitter language handle from the vendored parser.
    fn tree_sitter_sql() -> *const ();
}

const MARKDOWN_LANGUAGE: tree_sitter_language::LanguageFn = tree_sitter_md::LANGUAGE;
const MARKDOWN_INLINE_LANGUAGE: tree_sitter_language::LanguageFn = tree_sitter_md::INLINE_LANGUAGE;
const MARKDOWN_HIGHLIGHTS_QUERY: &str = tree_sitter_md::HIGHLIGHT_QUERY_BLOCK;
const MARKDOWN_INJECTIONS_QUERY: &str = tree_sitter_md::INJECTION_QUERY_BLOCK;
const MARKDOWN_INLINE_HIGHLIGHTS_QUERY: &str = tree_sitter_md::HIGHLIGHT_QUERY_INLINE;
const MARKDOWN_INLINE_INJECTIONS_QUERY: &str = tree_sitter_md::INJECTION_QUERY_INLINE;
const XML_LANGUAGE: tree_sitter_language::LanguageFn = tree_sitter_xml::LANGUAGE_XML;
const XML_HIGHLIGHTS_QUERY: &str = tree_sitter_xml::XML_HIGHLIGHT_QUERY;
const XML_IMPLEMENTATION_INJECTIONS_QUERY: &str = r#"
(
  element
    (STag (Name) @_start_tag)
    (content (CDSect (CData) @injection.content))
    (ETag (Name) @_end_tag)
  (#eq? @_start_tag "Implementation")
  (#eq? @_end_tag "Implementation")
  (#set! injection.language "objectscript")
)
(
  element
    (STag (Name) @_start_tag)
    (content (CharData) @injection.content)
    (ETag (Name) @_end_tag)
  (#eq? @_start_tag "Implementation")
  (#eq? @_end_tag "Implementation")
  (#set! injection.language "objectscript")
)
"#;

const SQL_LANGUAGE: tree_sitter_language::LanguageFn =
    unsafe { tree_sitter_language::LanguageFn::from_raw(tree_sitter_sql) };
const SQL_HIGHLIGHTS_QUERY: &str = include_str!("../vendor/tree-sitter-sql/queries/highlights.scm");

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Grammar {
    ObjectScript,
    Sql,
    Python,
    Markdown,
    Mdx,
    Xml,
}

const SUPPORTED_GRAMMARS: [&str; 6] = ["objectscript", "sql", "python", "markdown", "mdx", "xml"];

impl Grammar {
    /// Parses a grammar name or alias into a [`Grammar`] value.
    ///
    /// The input is normalized to lowercase alphanumeric characters, so values
    /// such as `"ObjectScript"`, `"objectscript-playground"`, and `"os"` are accepted.
    #[must_use]
    pub fn from_name(input: &str) -> Option<Self> {
        let normalized = normalize_language_name(input);
        grammar_from_normalized_name(&normalized)
    }

    /// Returns the canonical lowercase name for this grammar.
    #[must_use]
    pub fn canonical_name(self) -> &'static str {
        match self {
            Self::ObjectScript => "objectscript",
            Self::Sql => "sql",
            Self::Python => "python",
            Self::Markdown => "markdown",
            Self::Mdx => "mdx",
            Self::Xml => "xml",
        }
    }

    /// Returns the canonical grammar names accepted by the CLI-facing APIs.
    #[must_use]
    pub fn supported_names() -> &'static [&'static str] {
        &SUPPORTED_GRAMMARS
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Attr {
    pub id: usize,
    pub capture_name: String,
}

impl Attr {
    /// Returns the theme lookup key for this capture (for example `"@keyword"`).
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
    #[error("failed to configure parser language: {0}")]
    Language(#[from] tree_sitter::LanguageError),
    #[error("failed to parse source for injection analysis")]
    Parse,
}

pub struct SpanHighlighter {
    highlighter: TsHighlighter,
    attrs: Vec<Attr>,
    objectscript: HighlightConfiguration,
    sql: HighlightConfiguration,
    python: HighlightConfiguration,
    markdown: HighlightConfiguration,
    markdown_inline: HighlightConfiguration,
    xml: HighlightConfiguration,
    objectscript_injection_query: tree_sitter::Query,
    objectscript_injection_content_capture: Option<u32>,
    objectscript_injection_language_capture: Option<u32>,
    xml_injection_query: tree_sitter::Query,
    xml_injection_content_capture: Option<u32>,
    xml_injection_language_capture: Option<u32>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct InjectionRegion {
    grammar: Grammar,
    start_byte: usize,
    end_byte: usize,
}

impl SpanHighlighter {
    /// Creates a highlighter configured for all supported grammars and injections.
    ///
    /// This preloads Tree-sitter highlight configurations for ObjectScript, SQL,
    /// Python, and Markdown variants, and builds a unified capture table.
    ///
    /// # Errors
    ///
    /// Returns an error if any grammar query cannot be compiled or if parser
    /// language configuration fails.
    pub fn new() -> Result<Self, HighlightError> {
        let objectscript_language: tree_sitter::Language =
            tree_sitter_objectscript_playground::LANGUAGE_OBJECTSCRIPT.into();
        let mut objectscript = new_config(
            objectscript_language.clone(),
            "objectscript",
            tree_sitter_objectscript_playground::HIGHLIGHTS_QUERY,
            tree_sitter_objectscript_playground::INJECTIONS_QUERY,
        )?;
        let mut sql = new_config(SQL_LANGUAGE.into(), "sql", SQL_HIGHLIGHTS_QUERY, "")?;
        let mut python = new_config(
            tree_sitter_python::LANGUAGE.into(),
            "python",
            tree_sitter_python::HIGHLIGHTS_QUERY,
            "",
        )?;
        let mut markdown = new_config(
            MARKDOWN_LANGUAGE.into(),
            "markdown",
            MARKDOWN_HIGHLIGHTS_QUERY,
            MARKDOWN_INJECTIONS_QUERY,
        )?;
        let mut markdown_inline = new_config(
            MARKDOWN_INLINE_LANGUAGE.into(),
            "markdown_inline",
            MARKDOWN_INLINE_HIGHLIGHTS_QUERY,
            MARKDOWN_INLINE_INJECTIONS_QUERY,
        )?;
        let xml_language: tree_sitter::Language = XML_LANGUAGE.into();
        let mut xml = new_config(xml_language.clone(), "xml", XML_HIGHLIGHTS_QUERY, "")?;
        let objectscript_injection_query = tree_sitter::Query::new(
            &objectscript_language,
            tree_sitter_objectscript_playground::INJECTIONS_QUERY,
        )?;
        let (objectscript_injection_content_capture, objectscript_injection_language_capture) =
            injection_capture_indices(&objectscript_injection_query);
        let xml_injection_query =
            tree_sitter::Query::new(&xml_language, XML_IMPLEMENTATION_INJECTIONS_QUERY)?;
        let (xml_injection_content_capture, xml_injection_language_capture) =
            injection_capture_indices(&xml_injection_query);

        let mut recognized = Vec::<String>::new();
        let mut capture_index_by_name = HashMap::<String, usize>::new();
        for config in [
            &objectscript,
            &sql,
            &python,
            &markdown,
            &markdown_inline,
            &xml,
        ] {
            for name in config.names() {
                if capture_index_by_name.contains_key(*name) {
                    continue;
                }
                let id = recognized.len();
                let owned = (*name).to_string();
                capture_index_by_name.insert(owned.clone(), id);
                recognized.push(owned);
            }
        }
        let recognized_refs = recognized.iter().map(String::as_str).collect::<Vec<_>>();
        objectscript.configure(&recognized_refs);
        sql.configure(&recognized_refs);
        python.configure(&recognized_refs);
        markdown.configure(&recognized_refs);
        markdown_inline.configure(&recognized_refs);
        xml.configure(&recognized_refs);
        let attrs = recognized
            .into_iter()
            .enumerate()
            .map(|(id, capture_name)| Attr { id, capture_name })
            .collect::<Vec<_>>();

        Ok(Self {
            highlighter: TsHighlighter::new(),
            attrs,
            objectscript,
            sql,
            python,
            markdown,
            markdown_inline,
            xml,
            objectscript_injection_query,
            objectscript_injection_content_capture,
            objectscript_injection_language_capture,
            xml_injection_query,
            xml_injection_content_capture,
            xml_injection_language_capture,
        })
    }

    /// Highlights a source buffer and returns capture attributes plus byte spans.
    ///
    /// When `flavor` is [`Grammar::ObjectScript`], language injections are resolved
    /// and applied to injected regions (for example embedded SQL blocks). When
    /// `flavor` is [`Grammar::Xml`], ObjectScript injections are applied to
    /// recognized XML embedded-code regions (for example `<Implementation>` bodies).
    ///
    /// # Errors
    ///
    /// Returns an error if Tree-sitter highlighting fails or if injection parsing
    /// cannot be completed.
    pub fn highlight(
        &mut self,
        source: &[u8],
        flavor: Grammar,
    ) -> Result<HighlightResult, HighlightError> {
        let mut result = self.highlight_base(source, flavor)?;
        if flavor == Grammar::ObjectScript {
            self.apply_objectscript_injections(source, &mut result)?;
        } else if flavor == Grammar::Xml {
            self.apply_xml_injections(source, &mut result)?;
        }
        Ok(result)
    }

    /// Runs the base Tree-sitter highlight pass for a single grammar.
    ///
    /// Unlike [`Self::highlight`], this does not apply post-processing for
    /// host-language injection regions.
    ///
    /// # Errors
    ///
    /// Returns an error if Tree-sitter fails to emit highlight events.
    fn highlight_base(
        &mut self,
        source: &[u8],
        flavor: Grammar,
    ) -> Result<HighlightResult, HighlightError> {
        let config = match flavor {
            Grammar::ObjectScript => &self.objectscript,
            Grammar::Sql => &self.sql,
            Grammar::Python => &self.python,
            Grammar::Markdown => &self.markdown,
            // InterSystems MDX is OLAP query syntax; use SQL highlighting as a temporary fallback.
            Grammar::Mdx => &self.sql,
            Grammar::Xml => &self.xml,
        };

        let attrs = self.attrs.clone();

        let injections = InjectionConfigs {
            objectscript: &self.objectscript,
            sql: &self.sql,
            python: &self.python,
            markdown: &self.markdown,
            markdown_inline: &self.markdown_inline,
            xml: &self.xml,
        };

        let events = self
            .highlighter
            .highlight(config, source, None, move |language_name| {
                injections.resolve(language_name)
            })?;
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

    /// Highlights line-oriented input by joining lines with `\n`.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::highlight`].
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

    /// Replaces ObjectScript injection regions in `base` with injected highlights.
    ///
    /// This method removes spans from injected byte ranges and merges spans produced
    /// by the injected language highlighter.
    ///
    /// # Errors
    ///
    /// Returns an error if injection discovery or nested highlighting fails.
    fn apply_objectscript_injections(
        &mut self,
        source: &[u8],
        base: &mut HighlightResult,
    ) -> Result<(), HighlightError> {
        let injections = self.find_objectscript_injections(source)?;
        self.apply_injections(source, base, injections)
    }

    /// Replaces XML injection regions in `base` with injected highlights.
    ///
    /// This currently targets XML regions where ObjectScript appears in
    /// `<Implementation>` bodies.
    fn apply_xml_injections(
        &mut self,
        source: &[u8],
        base: &mut HighlightResult,
    ) -> Result<(), HighlightError> {
        let injections = self.find_xml_injections(source)?;
        self.apply_injections(source, base, injections)
    }

    /// Applies already-discovered injection regions by replacing base spans.
    fn apply_injections(
        &mut self,
        source: &[u8],
        base: &mut HighlightResult,
        injections: Vec<InjectionRegion>,
    ) -> Result<(), HighlightError> {
        if injections.is_empty() {
            return Ok(());
        }

        let mut attrs = base.attrs.clone();
        let mut attr_ids_by_name = attrs
            .iter()
            .map(|attr| (attr.capture_name.clone(), attr.id))
            .collect::<HashMap<_, _>>();
        let mut injected_spans = Vec::new();

        for injection in &injections {
            let nested_source = &source[injection.start_byte..injection.end_byte];
            let nested = self.highlight_base(nested_source, injection.grammar)?;
            let remap = remap_attr_ids(&nested.attrs, &mut attrs, &mut attr_ids_by_name);
            for span in nested.spans {
                let Some(&mapped_attr_id) = remap.get(span.attr_id) else {
                    continue;
                };
                injected_spans.push(Span {
                    attr_id: mapped_attr_id,
                    start_byte: span.start_byte + injection.start_byte,
                    end_byte: span.end_byte + injection.start_byte,
                });
            }
        }

        let mut spans = exclude_ranges(
            &base.spans,
            &injections
                .iter()
                .map(|inj| (inj.start_byte, inj.end_byte))
                .collect::<Vec<_>>(),
        );
        spans.extend(injected_spans);

        base.attrs = attrs;
        base.spans = normalize_spans(spans);
        Ok(())
    }

    /// Finds non-overlapping ObjectScript injection regions in the source buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing or query execution for injection analysis fails.
    fn find_objectscript_injections(
        &self,
        source: &[u8],
    ) -> Result<Vec<InjectionRegion>, HighlightError> {
        let objectscript_language: tree_sitter::Language =
            tree_sitter_objectscript_playground::LANGUAGE_OBJECTSCRIPT.into();
        self.find_injections(
            source,
            &objectscript_language,
            &self.objectscript_injection_query,
            self.objectscript_injection_content_capture,
            self.objectscript_injection_language_capture,
        )
    }

    /// Finds non-overlapping XML injection regions in the source buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing or query execution for injection analysis fails.
    fn find_xml_injections(&self, source: &[u8]) -> Result<Vec<InjectionRegion>, HighlightError> {
        let xml_language: tree_sitter::Language = XML_LANGUAGE.into();
        self.find_injections(
            source,
            &xml_language,
            &self.xml_injection_query,
            self.xml_injection_content_capture,
            self.xml_injection_language_capture,
        )
    }

    /// Finds and normalizes non-overlapping injection regions for a host grammar.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing or query execution for injection analysis fails.
    fn find_injections(
        &self,
        source: &[u8],
        language: &tree_sitter::Language,
        query: &tree_sitter::Query,
        content_capture: Option<u32>,
        language_capture: Option<u32>,
    ) -> Result<Vec<InjectionRegion>, HighlightError> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(language)?;
        let tree = parser.parse(source, None).ok_or(HighlightError::Parse)?;
        let mut cursor = tree_sitter::QueryCursor::new();

        let mut injections = Vec::new();
        let mut matches = cursor.matches(query, tree.root_node(), source);
        while let Some(mat) = matches.next() {
            let Some(injection) = self.injection_region_for_match(
                query,
                content_capture,
                language_capture,
                source,
                &mat,
            ) else {
                continue;
            };
            injections.push(injection);
        }

        if injections.is_empty() {
            return Ok(injections);
        }

        injections.sort_by(|a, b| {
            a.start_byte
                .cmp(&b.start_byte)
                .then(b.end_byte.cmp(&a.end_byte))
                .then((a.grammar as u8).cmp(&(b.grammar as u8)))
        });
        injections.dedup_by(|a, b| {
            a.grammar == b.grammar && a.start_byte == b.start_byte && a.end_byte == b.end_byte
        });

        let mut non_overlapping = Vec::with_capacity(injections.len());
        let mut last_end = 0usize;
        for injection in injections {
            if injection.start_byte < last_end {
                continue;
            }
            last_end = injection.end_byte;
            non_overlapping.push(injection);
        }
        Ok(non_overlapping)
    }

    /// Converts a query match to an [`InjectionRegion`] when captures are complete.
    ///
    /// Returns `None` when language or content captures are missing, unknown, or empty.
    fn injection_region_for_match<'a>(
        &self,
        query: &tree_sitter::Query,
        content_capture: Option<u32>,
        language_capture: Option<u32>,
        source: &'a [u8],
        mat: &tree_sitter::QueryMatch<'a, 'a>,
    ) -> Option<InjectionRegion> {
        let mut language_name = None;
        let mut content_node = None;

        for capture in mat.captures {
            let index = Some(capture.index);
            if index == language_capture {
                language_name = capture.node.utf8_text(source).ok();
            } else if index == content_capture {
                content_node = Some(capture.node);
            }
        }

        for prop in query.property_settings(mat.pattern_index) {
            match prop.key.as_ref() {
                "injection.language" => {
                    if language_name.is_none() {
                        language_name = prop.value.as_ref().map(std::convert::AsRef::as_ref);
                    }
                }
                "injection.self" | "injection.parent" => {
                    if language_name.is_none() {
                        language_name = Some("objectscript");
                    }
                }
                _ => {}
            }
        }

        let grammar = language_name.and_then(Grammar::from_name)?;
        let content_node = content_node?;
        let start_byte = content_node.start_byte();
        let end_byte = content_node.end_byte();
        if start_byte >= end_byte {
            return None;
        }

        Some(InjectionRegion {
            grammar,
            start_byte,
            end_byte,
        })
    }
}

struct InjectionConfigs<'a> {
    objectscript: &'a HighlightConfiguration,
    sql: &'a HighlightConfiguration,
    python: &'a HighlightConfiguration,
    markdown: &'a HighlightConfiguration,
    markdown_inline: &'a HighlightConfiguration,
    xml: &'a HighlightConfiguration,
}

impl<'a> InjectionConfigs<'a> {
    /// Resolves an injected language name to a highlight configuration.
    ///
    /// Unknown language names return `None` so Tree-sitter skips injection highlighting.
    fn resolve(&self, language_name: &str) -> Option<&'a HighlightConfiguration> {
        let normalized = normalize_language_name(language_name);
        if normalized == "markdowninline" {
            return Some(self.markdown_inline);
        }

        let grammar = grammar_from_normalized_name(&normalized)?;
        match grammar {
            Grammar::ObjectScript => Some(self.objectscript),
            Grammar::Sql => Some(self.sql),
            Grammar::Python => Some(self.python),
            Grammar::Markdown => Some(self.markdown),
            Grammar::Mdx => Some(self.sql),
            Grammar::Xml => Some(self.xml),
        }
    }
}

/// Normalizes a language name by retaining only ASCII alphanumerics and
/// lowercasing the result.
fn normalize_language_name(input: &str) -> String {
    input
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

/// Maps a normalized language name to a supported [`Grammar`].
fn grammar_from_normalized_name(normalized: &str) -> Option<Grammar> {
    match normalized {
        "objectscript" | "os" | "playground" | "objectscriptplayground" => {
            Some(Grammar::ObjectScript)
        }
        "sql" | "tsql" | "plsql" | "mysql" | "postgres" | "postgresql" => Some(Grammar::Sql),
        "python" | "py" => Some(Grammar::Python),
        "markdown" | "md" | "gfm" => Some(Grammar::Markdown),
        "mdx" => Some(Grammar::Mdx),
        "xml" => Some(Grammar::Xml),
        _ => None,
    }
}

/// Locates `injection.content` and `injection.language` captures in a query.
fn injection_capture_indices(query: &tree_sitter::Query) -> (Option<u32>, Option<u32>) {
    let mut content_capture = None;
    let mut language_capture = None;
    for (idx, name) in query.capture_names().iter().enumerate() {
        let idx = Some(idx as u32);
        match *name {
            "injection.content" => content_capture = idx,
            "injection.language" => language_capture = idx,
            _ => {}
        }
    }
    (content_capture, language_capture)
}

/// Builds and configures a Tree-sitter highlight configuration.
///
/// # Errors
///
/// Returns an error when the highlight or injection query is invalid for the
/// provided language.
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

/// Pushes a span into `spans`, merging with the previous span when adjacent and
/// sharing the same attribute id.
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

/// Remaps incoming attribute ids to ids in the destination attribute table.
///
/// Existing destination ids are reused by capture name; new capture names are appended.
fn remap_attr_ids(
    incoming: &[Attr],
    attrs: &mut Vec<Attr>,
    attr_ids_by_name: &mut HashMap<String, usize>,
) -> Vec<usize> {
    let mut remap = vec![0usize; incoming.len()];
    for attr in incoming {
        let mapped_attr_id = if let Some(&mapped_attr_id) = attr_ids_by_name.get(&attr.capture_name)
        {
            mapped_attr_id
        } else {
            let mapped_attr_id = attrs.len();
            attrs.push(Attr {
                id: mapped_attr_id,
                capture_name: attr.capture_name.clone(),
            });
            attr_ids_by_name.insert(attr.capture_name.clone(), mapped_attr_id);
            mapped_attr_id
        };
        if let Some(slot) = remap.get_mut(attr.id) {
            *slot = mapped_attr_id;
        }
    }
    remap
}

/// Removes byte `ranges` from `spans`, splitting spans as needed.
fn exclude_ranges(spans: &[Span], ranges: &[(usize, usize)]) -> Vec<Span> {
    if ranges.is_empty() {
        return spans.to_vec();
    }

    let mut out: Vec<Span> = Vec::with_capacity(spans.len());
    let mut range_idx = 0usize;
    for span in spans {
        while range_idx < ranges.len() && ranges[range_idx].1 <= span.start_byte {
            range_idx += 1;
        }

        let mut cursor = span.start_byte;
        let mut idx = range_idx;
        while idx < ranges.len() {
            let (range_start, range_end) = ranges[idx];
            if range_start >= span.end_byte {
                break;
            }

            if range_end <= cursor {
                idx += 1;
                continue;
            }

            if cursor < range_start {
                push_merged(
                    &mut out,
                    Span {
                        attr_id: span.attr_id,
                        start_byte: cursor,
                        end_byte: range_start.min(span.end_byte),
                    },
                );
            }

            if range_end >= span.end_byte {
                cursor = span.end_byte;
                break;
            }

            cursor = range_end;
            idx += 1;
        }

        if cursor < span.end_byte {
            push_merged(
                &mut out,
                Span {
                    attr_id: span.attr_id,
                    start_byte: cursor,
                    end_byte: span.end_byte,
                },
            );
        }
    }
    out
}

/// Sorts spans and enforces a non-overlapping, merge-friendly representation.
fn normalize_spans(mut spans: Vec<Span>) -> Vec<Span> {
    spans.sort_by(|a, b| {
        a.start_byte
            .cmp(&b.start_byte)
            .then(a.end_byte.cmp(&b.end_byte))
            .then(a.attr_id.cmp(&b.attr_id))
    });

    let mut out: Vec<Span> = Vec::with_capacity(spans.len());
    for mut span in spans {
        if let Some(last) = out.last() {
            if span.start_byte < last.end_byte {
                if span.end_byte <= last.end_byte {
                    continue;
                }
                span.start_byte = last.end_byte;
            }
        }
        push_merged(&mut out, span);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Grammar, HighlightResult, SpanHighlighter};

    /// Returns whether `expected_text` appears under `capture_name` in `result`.
    fn has_capture_for_text(
        result: &HighlightResult,
        source: &[u8],
        capture_name: &str,
        expected_text: &[u8],
    ) -> bool {
        let attr_id = match result
            .attrs
            .iter()
            .find(|attr| attr.capture_name == capture_name)
            .map(|attr| attr.id)
        {
            Some(id) => id,
            None => return false,
        };

        result.spans.iter().any(|span| {
            span.attr_id == attr_id && &source[span.start_byte..span.end_byte] == expected_text
        })
    }

    #[test]
    /// Verifies ObjectScript numeric literals are tagged as `number`.
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

        assert!(
            has_capture_for_text(&result, source, "number", b"42"),
            "expected highlighted span for numeric literal"
        );
    }

    #[test]
    /// Verifies canonical and alias grammar names resolve correctly.
    fn parses_supported_grammar_aliases() {
        assert_eq!(
            Grammar::from_name("objectscript"),
            Some(Grammar::ObjectScript)
        );
        assert_eq!(Grammar::from_name("SQL"), Some(Grammar::Sql));
        assert_eq!(Grammar::from_name("py"), Some(Grammar::Python));
        assert_eq!(Grammar::from_name("md"), Some(Grammar::Markdown));
        assert_eq!(Grammar::from_name("mdx"), Some(Grammar::Mdx));
        assert_eq!(Grammar::from_name("xml"), Some(Grammar::Xml));
        assert!(Grammar::from_name("unknown").is_none());
    }

    #[test]
    /// Verifies SQL keywords are captured as `keyword`.
    fn highlights_sql_keyword() {
        let source = b"SELECT 42 FROM Demo";
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::Sql)
            .expect("failed to highlight SQL");

        assert!(
            has_capture_for_text(&result, source, "keyword", b"SELECT"),
            "expected SELECT to be highlighted as keyword"
        );
    }

    #[test]
    /// Verifies `%SQLQuery` bodies are highlighted via SQL injection handling.
    fn objectscript_sqlquery_body_is_highlighted_as_sql() {
        let source = br#"
Class Test
{
  Query ListEmployees() As %SQLQuery
  {
SELECT ID,Name FROM Employee
  }
}
"#;
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::ObjectScript)
            .expect("failed to highlight ObjectScript with SQL injection");

        assert!(
            has_capture_for_text(&result, source, "keyword", b"SELECT"),
            "expected SQL SELECT in %SQLQuery body to be highlighted as keyword"
        );
    }

    #[test]
    /// Verifies Python numeric literals are highlighted as `number`.
    fn highlights_python_number() {
        let source = b"def f(x):\n    return x + 1\n";
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::Python)
            .expect("failed to highlight Python");

        assert!(
            has_capture_for_text(&result, source, "number", b"1"),
            "expected numeric literal to be highlighted in Python"
        );
    }

    #[test]
    /// Verifies Markdown heading text is captured as `text.title`.
    fn highlights_markdown_heading() {
        let source = b"# Heading\n";
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::Markdown)
            .expect("failed to highlight Markdown");

        assert!(
            has_capture_for_text(&result, source, "text.title", b"Heading"),
            "expected heading text to be highlighted in Markdown"
        );
    }

    #[test]
    /// Verifies MDX currently falls back to SQL keyword highlighting.
    fn mdx_falls_back_to_sql_keyword_highlighting() {
        let source = b"SELECT 1 FROM Cube";
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::Mdx)
            .expect("failed to highlight MDX fallback");

        assert!(
            has_capture_for_text(&result, source, "keyword", b"SELECT"),
            "expected MDX fallback to highlight SQL keywords"
        );
    }

    #[test]
    /// Verifies ObjectScript inside XML `<Implementation>` CDATA is injected.
    fn xml_implementation_cdata_is_highlighted_as_objectscript() {
        let source = br#"
<Export>
  <Class name="Demo.Sample">
    <Method name="Run">
      <Implementation><![CDATA[
 set x = 42
]]></Implementation>
    </Method>
  </Class>
</Export>
"#;
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::Xml)
            .expect("failed to highlight XML with ObjectScript injection");

        assert!(
            has_capture_for_text(&result, source, "number", b"42"),
            "expected injected ObjectScript numeric literal to be highlighted"
        );
    }
}
