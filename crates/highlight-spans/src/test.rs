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
            has_capture_for_text(&result, source, "function.method", b"proc"),
            "expected highlighted span to be function.method for proc"
        );
        assert!(
            has_capture_for_text(&result, source, "punctuation.delimiter", b"["),
            "expected highlighted span to be punctuation.delimiter for ["
        );
        assert!(
            has_capture_for_text(&result, source, "type.builtin", b"type "),
            "expected highlighted span to be type.builtin for type, but instead it was {:?}",
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
    fn objectscript_yaml_xdata_is_highlighted() {
        let source = br#"Class Test
{
  XData SampleAPI [mimetype = "application/yaml"]
  {
swagger: "2.0"
info:
  title: Sample API
  }
}"#;
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::ObjectScript)
            .expect("failed to highlight ObjectScript with YAML injection");

        assert!(
            has_capture_for_text(&result, source, "string", b"\"2.0\""),
            "expected YAML string value to be highlighted in YAML XData injection"
        );
        assert!(
            has_capture_for_text(&result, source, "property", b"swagger"),
            "expected YAML key to be highlighted as property in YAML XData injection"
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

    #[test]
    fn yaml_standalone_highlights_correctly() {
        let source = b"swagger: \"2.0\"\ninfo:\n  title: Sample API\n";
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::Yaml)
            .expect("failed to highlight YAML");

        eprintln!("\n--- YAML standalone spans ---");
        for span in &result.spans {
            let text =
                std::str::from_utf8(&source[span.start_byte..span.end_byte]).unwrap_or("<binary>");
            let capture_name = &result.attrs[span.attr_id].capture_name;
            eprintln!(
                "  [{}..{}] {:25} {:?}",
                span.start_byte, span.end_byte, capture_name, text
            );
        }

        assert!(
            has_capture_for_text(&result, source, "string", b"\"2.0\""),
            "expected YAML string"
        );
    }

    #[test]
    fn diff_bare_unified_diff_debug() {
        let source = b"--- a/foo.py\n+++ b/foo.py\n@@ -1,2 +1,2 @@\n-import os\n+import sys\n def main():\n";
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::Diff)
            .expect("failed to highlight diff");

        eprintln!("\n--- bare unified diff spans ---");
        for span in &result.spans {
            let text =
                std::str::from_utf8(&source[span.start_byte..span.end_byte]).unwrap_or("<binary>");
            let capture_name = &result.attrs[span.attr_id].capture_name;
            eprintln!(
                "  [{}..{}] {:25} {:?}",
                span.start_byte, span.end_byte, capture_name, text
            );
        }
    }

    #[test]
    fn diff_git_style_with_python_injection() {
        let source = b"diff --git a/foo.py b/foo.py\nindex 1234567..89abcde 100644\n--- a/foo.py\n+++ b/foo.py\n@@ -1,2 +1,2 @@\n-import os\n+import sys\n def main():\n";
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::Diff)
            .expect("failed to highlight diff");

        eprintln!("\n--- git-style diff spans (expect Python injection) ---");
        for span in &result.spans {
            let text =
                std::str::from_utf8(&source[span.start_byte..span.end_byte]).unwrap_or("<binary>");
            let capture_name = &result.attrs[span.attr_id].capture_name;
            eprintln!(
                "  [{}..{}] {:25} {:?}",
                span.start_byte, span.end_byte, capture_name, text
            );
        }

        assert!(
            has_capture_for_text(&result, source, "keyword", b"import"),
            "expected an injected Python 'import' keyword inside a diff hunk line"
        );
    }

    #[test]
    fn diff_plus_marker_carries_the_line_background_even_though_punctuation_special_wins_the_byte(
    ) {
        let source = b"@@ -1,1 +1,1 @@\n+added line\n";
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::Diff)
            .expect("failed to highlight diff");

        let diff_plus_attr_id = result
            .attrs
            .iter()
            .find(|attr| attr.capture_name == "diff.plus")
            .map(|attr| attr.id)
            .expect("expected a diff.plus attribute");

        let marker_byte = source
            .windows(b"+added".len())
            .position(|w| w == b"+added")
            .expect("marker byte");
        let marker_span = result
            .spans
            .iter()
            .find(|span| span.start_byte <= marker_byte && marker_byte < span.end_byte)
            .expect("expected a span covering the '+' marker");

        assert_eq!(
            result.attrs[marker_span.attr_id].capture_name,
            "punctuation.special",
            "sanity check: the marker byte should still be won by punctuation.special"
        );
        assert_eq!(
            marker_span.background_attr_id,
            Some(diff_plus_attr_id),
            "the marker span should carry diff.plus's background even though \
             punctuation.special won the byte outright"
        );
    }

    /// Returns the capture name of the ambient background on the span whose
    /// text is exactly `expected_text` under `capture_name`, if any.
    fn background_for_text(
        result: &HighlightResult,
        source: &[u8],
        capture_name: &str,
        expected_text: &[u8],
    ) -> Option<String> {
        result
            .spans
            .iter()
            .find(|span| {
                result.attrs[span.attr_id].capture_name == capture_name
                    && &source[span.start_byte..span.end_byte] == expected_text
            })
            .and_then(|span| span.background_attr_id)
            .map(|id| result.attrs[id].capture_name.clone())
    }

    #[test]
    /// Verifies `&sql()` inside a UDL class is injected as SQL.
    fn udl_embedded_sql_is_highlighted_as_sql() {
        let source = b"Class A.B\n{\nClassMethod M()\n{\n  &sql(SELECT Name FROM T)\n}\n}\n";
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::ObjectScriptUdl)
            .expect("failed to highlight");

        assert!(
            has_capture_for_text(&result, source, "keyword", b"SELECT"),
            "expected SQL keyword inside UDL &sql()"
        );
    }

    #[test]
    /// Verifies HTML `<script>` and `<style>` bodies are injected as JavaScript and CSS.
    fn html_script_and_style_are_injected() {
        let source = b"<html><script>const x = 1;</script><style>body { color: red; }</style></html>\n";
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::Html)
            .expect("failed to highlight");

        assert!(
            has_capture_for_text(&result, source, "keyword", b"const"),
            "expected JavaScript keyword inside <script>"
        );
        assert!(
            has_capture_for_text(&result, source, "property", b"color"),
            "expected CSS property inside <style>"
        );
    }

    #[test]
    /// Verifies JavaScript doc comments are injected as JSDoc.
    fn javascript_jsdoc_comment_is_injected() {
        let source = b"/** @param {number} x */\nfunction f(x) {}\n";
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::JavaScript)
            .expect("failed to highlight");

        assert!(
            has_capture_for_text(&result, source, "keyword", b"@param"),
            "expected JSDoc tag inside a /** */ comment"
        );
    }

    #[test]
    /// Verifies Markdown fenced code and inline content are both injected.
    fn markdown_fenced_code_and_inline_are_injected() {
        let source = b"some **bold** text\n\n```python\ndef f():\n    return 1\n```\n";
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::Markdown)
            .expect("failed to highlight");

        assert!(
            has_capture_for_text(&result, source, "text.strong", b"bold"),
            "expected inline Markdown emphasis"
        );
        assert!(
            has_capture_for_text(&result, source, "keyword", b"def"),
            "expected Python keyword inside a ```python fence"
        );
    }

    #[test]
    /// Verifies injections nest: Markdown -> HTML block -> JavaScript.
    fn markdown_html_block_script_is_injected_two_levels_deep() {
        let source = b"# T\n\n<div>\n<script>const x = 1;</script>\n</div>\n";
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::Markdown)
            .expect("failed to highlight");

        assert!(
            has_capture_for_text(&result, source, "keyword", b"const"),
            "expected JavaScript keyword inside a <script> in a Markdown HTML block"
        );
    }

    #[test]
    /// Verifies a nested injection keeps the outermost ambient background:
    /// diff -> Markdown -> inline Markdown still sits on `diff.plus`.
    fn diff_background_survives_nested_markdown_inline_injection() {
        let source = b"diff --git a/x.md b/x.md\nindex 1234567..89abcde 100644\n--- a/x.md\n+++ b/x.md\n@@ -1,2 +1,2 @@\n-old\n+some **bold** text\n";
        let mut highlighter = SpanHighlighter::new().expect("failed to build highlighter");
        let result = highlighter
            .highlight(source, Grammar::Diff)
            .expect("failed to highlight diff");

        assert_eq!(
            background_for_text(&result, source, "text.strong", b"bold").as_deref(),
            Some("diff.plus"),
            "expected inline emphasis inside a + line to carry the diff.plus background"
        );
    }
}
