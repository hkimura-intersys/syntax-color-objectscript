use std::collections::HashMap;

use thiserror::Error;
use tree_sitter::{Parser, Query};

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
    pub(crate) configs: HashMap<Grammar, HighlightConfig>,
    pub(crate) attrs: Vec<Attr>,
    pub(crate) attr_ids_by_name: HashMap<String, usize>,
    pub(crate) timing_logs_enabled: bool,
}

pub(crate) struct HighlightConfig {
    pub(crate) parser: Parser,
    pub(crate) highlight_query: Query,
    pub(crate) injection_query: Option<Query>,
    pub(crate) injection_content_capture_index: Option<u32>,
    pub(crate) injection_language_capture_index: Option<u32>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Grammar {
    ObjectScript,
    ObjectScriptRoutine,
    Sql,
    Python,
    Markdown,
    Mdx,
    Xml,
    Json,
    Yaml,
    Css,
    Html,
    JavaScript,
    JsDoc,
    ObjectScriptUdl,
    Regex,
    Toml,
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Attr {
    pub id: usize,
    pub capture_name: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct InjectionRegion {
    pub grammar: Grammar,
    pub start_byte: usize,
    pub end_byte: usize,
}

impl Attr {
    /// Returns the theme lookup key for this capture (for example `"@keyword"`).
    #[must_use]
    pub fn theme_key(&self) -> String {
        format!("@{}", self.capture_name)
    }
}

unsafe extern "C" {
    /// Returns the SQL Tree-sitter language handle from the vendored parser.
    fn tree_sitter_sql() -> *const ();
}

pub const MARKDOWN_LANGUAGE: tree_sitter_language::LanguageFn = tree_sitter_md::LANGUAGE;
pub const MARKDOWN_INLINE_LANGUAGE: tree_sitter_language::LanguageFn =
    tree_sitter_md::INLINE_LANGUAGE;
pub const MARKDOWN_HIGHLIGHTS_QUERY: &str = tree_sitter_md::HIGHLIGHT_QUERY_BLOCK;
pub const MARKDOWN_INJECTIONS_QUERY: &str = tree_sitter_md::INJECTION_QUERY_BLOCK;
pub const MARKDOWN_INLINE_HIGHLIGHTS_QUERY: &str = tree_sitter_md::HIGHLIGHT_QUERY_INLINE;
pub const MARKDOWN_INLINE_INJECTIONS_QUERY: &str = tree_sitter_md::INJECTION_QUERY_INLINE;
pub const JSON_LANGUAGE: tree_sitter_language::LanguageFn = tree_sitter_json::LANGUAGE;
pub const JSON_HIGHLIGHTS_QUERY: &str = tree_sitter_json::HIGHLIGHTS_QUERY;
pub const YAML_LANGUAGE: tree_sitter_language::LanguageFn = tree_sitter_yaml::LANGUAGE;
pub const YAML_HIGHLIGHTS_QUERY: &str = tree_sitter_yaml::HIGHLIGHTS_QUERY;
pub const XML_LANGUAGE: tree_sitter_language::LanguageFn = tree_sitter_xml::LANGUAGE_XML;
pub const XML_HIGHLIGHTS_QUERY: &str = tree_sitter_xml::XML_HIGHLIGHT_QUERY;
pub const XML_IMPLEMENTATION_INJECTIONS_QUERY: &str = r#"
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

pub const SQL_LANGUAGE: tree_sitter_language::LanguageFn =
    unsafe { tree_sitter_language::LanguageFn::from_raw(tree_sitter_sql) };
pub const SQL_HIGHLIGHTS_QUERY: &str =
    include_str!("../vendor/tree-sitter-sql/queries/highlights.scm");


pub const SUPPORTED_GRAMMARS: [&str; 16] = [
    "objectscript",
    "objectscript_routine",
    "sql",
    "python",
    "markdown",
    "mdx",
    "xml",
    "json",
    "yaml",
    "css",
    "html",
    "javascript",
    "jsdoc",
    "objectscript_udl",
    "regex",
    "toml",
];

#[derive(Debug)]
pub(crate) struct CaptureCandidate {
    pub(crate) attr_id: usize,
    pub(crate) start_byte: usize,
    pub(crate) end_byte: usize,
    pub(crate) render_priority: usize,
    pub(crate) specificity: usize,
    pub(crate) pattern_index: usize,
    pub(crate) order: usize,
}
