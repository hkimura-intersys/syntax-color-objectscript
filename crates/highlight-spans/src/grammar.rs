use crate::highlight_structures::*;

impl Grammar {
    /// Parses a grammar name or alias into a [`Grammar`] value.
    #[must_use]
    pub fn from_name(input: &str) -> Option<Self> {
        match input {
            "objectscript" | "cls" | "objectscriptplayground" | "objectscript_playground" => {
                Some(Self::ObjectScript)
            }
            "rtn"
            | "mac"
            | "int"
            | "inc"
            | "objectscript_routine"
            | "objectscript-routine"
            | "routine" => Some(Self::ObjectScriptRoutine),
            "sql" | "tsql" | "plsql" | "mysql" | "postgres" => Some(Self::Sql),
            "python" | "py" => Some(Self::Python),
            "markdown" | "md" | "gfm" => Some(Self::Markdown),
            "mdx" => Some(Self::Mdx),
            "xml" => Some(Self::Xml),
            "json" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            "css" => Some(Self::Css),
            "html" => Some(Self::Html),
            "javascript" | "js" => Some(Self::JavaScript),
            "jsdoc" => Some(Self::JsDoc),
            "objectscript_udl" | "objectscript-udl" | "udl" => Some(Self::ObjectScriptUdl),
            "regex" => Some(Self::Regex),
            "toml" => Some(Self::Toml),
            "diff" | "patch" | "udiff" => Some(Self::Diff),
            _ => None,
        }
    }

    /// Returns the canonical lowercase name for this grammar.
    #[must_use]
    pub fn canonical_name(self) -> &'static str {
        match self {
            Self::ObjectScript => "objectscript",
            Self::ObjectScriptRoutine => "objectscript_routine",
            Self::Sql => "sql",
            Self::Python => "python",
            Self::Markdown => "markdown",
            Self::Mdx => "mdx",
            Self::Xml => "xml",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Css => "css",
            Self::Html => "html",
            Self::JavaScript => "javascript",
            Self::JsDoc => "jsdoc",
            Self::ObjectScriptUdl => "objectscript_udl",
            Self::Regex => "regex",
            Self::Toml => "toml",
            Self::Diff => "diff",
        }
    }

    /// Returns the canonical grammar names accepted by the CLI-facing APIs.
    #[must_use]
    pub fn supported_names() -> &'static [&'static str] {
        &SUPPORTED_GRAMMARS
    }

    /// Derives a [`Grammar`] from a file path's extension (for example
    /// `"src/foo.py"` -> `Some(Self::Python)`), for resolving diff hunk
    /// injections captured via `@injection.filename` (see
    /// `tree-sitter-diff`'s `injections.scm`).
    ///
    /// Reuses [`Self::from_name`]'s own alias table rather than a second,
    /// separate extension map: most of its short aliases (`"py"`, `"md"`,
    /// `"yml"`, `"js"`, `"toml"`, `"cls"`, `"mac"`, ...) already double as
    /// common file extensions, so keeping one table avoids the two silently
    /// drifting out of sync.
    ///
    /// Returns `None` for an extension with no recognized grammar (for
    /// example `.rs`, `.go`) or a path with no extension at all.
    #[must_use]
    pub fn from_filename(path: &str) -> Option<Self> {
        let (_, ext) = path.rsplit_once('.')?;
        Self::from_name(&ext.to_ascii_lowercase())
    }
}
