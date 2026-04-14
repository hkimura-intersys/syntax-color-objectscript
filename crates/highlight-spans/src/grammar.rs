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
        }
    }

    /// Returns the canonical grammar names accepted by the CLI-facing APIs.
    #[must_use]
    pub fn supported_names() -> &'static [&'static str] {
        &SUPPORTED_GRAMMARS
    }
}
