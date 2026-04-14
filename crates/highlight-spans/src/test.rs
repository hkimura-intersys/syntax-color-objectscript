#[cfg(test)]
mod tests {
    use crate::highlight_structures::*;

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
    /// Verifies ObjectScript Routines are captured properly.
    fn highlights_routine_correctly() {
        let source = br#"Routine x [type = Mac]
proc() {
            set x = 2
        }
        "#;
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::ObjectScriptRoutine)
            .expect("failed to highlight");
        assert!(
            has_capture_for_text(&result, source, "keyword.type", b"Routine"),
            "expected highlighted span to be keyword.type for routine"
        );
        assert!(
            has_capture_for_text(&result, source, "label", b"proc"),
            "expected highlighted span to be label for proc"
        );
        assert!(
            has_capture_for_text(&result, source, "punctuation.special", b"["),
            "expected highlighted span to be punctuation.special for ["
        );
        assert!(
            has_capture_for_text(&result, source, "keyword.operator", b"type "),
            "expected highlighted span to be keword.operator for type, but instead it was {:?}",
            &result
        )
    }

    #[test]
    /// Verifies canonical and alias grammar names resolve correctly.
    fn parses_supported_grammar_aliases() {
        assert_eq!(
            Grammar::from_name("objectscript"),
            Some(Grammar::ObjectScript)
        );
        assert_eq!(Grammar::from_name("sql"), Some(Grammar::Sql));
        assert_eq!(Grammar::from_name("py"), Some(Grammar::Python));
        assert_eq!(Grammar::from_name("md"), Some(Grammar::Markdown));
        assert_eq!(Grammar::from_name("mdx"), Some(Grammar::Mdx));
        assert_eq!(Grammar::from_name("xml"), Some(Grammar::Xml));
        assert_eq!(Grammar::from_name("yaml"), Some(Grammar::Yaml));
        assert_eq!(Grammar::from_name("json"), Some(Grammar::Json));
        assert_eq!(Grammar::from_name("yml"), Some(Grammar::Yaml));
        assert_eq!(
            Grammar::from_name("objectscript_routine"),
            Some(Grammar::ObjectScriptRoutine)
        );
        assert_eq!(
            Grammar::from_name("rtn"),
            Some(Grammar::ObjectScriptRoutine)
        );
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
