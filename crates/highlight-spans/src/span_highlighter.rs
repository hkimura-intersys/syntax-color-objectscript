use crate::common::*;
use crate::highlight_structures::*;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::env;
use std::time::{Duration, Instant};
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};

const ALL_GRAMMARS: [Grammar; 16] = [
    Grammar::ObjectScript,
    Grammar::ObjectScriptRoutine,
    Grammar::Sql,
    Grammar::Python,
    Grammar::Markdown,
    Grammar::Mdx,
    Grammar::Xml,
    Grammar::Json,
    Grammar::Yaml,
    Grammar::Css,
    Grammar::Html,
    Grammar::JavaScript,
    Grammar::JsDoc,
    Grammar::ObjectScriptUdl,
    Grammar::Regex,
    Grammar::Toml,
];

const HIGHLIGHT_TIMING_ENV_VAR: &str = "SYNTAX_COLOR_LOG_HIGHLIGHT_TIMINGS";

#[derive(Clone, Copy, Default)]
struct HighlightTimingBreakdown {
    base: Duration,
    injection_query: Duration,
    injection_apply: Duration,
    injection_count: usize,
}

struct GrammarSource {
    language: Language,
    highlights_query: &'static str,
    injections_query: &'static str,
}

fn grammar_source(grammar: Grammar) -> GrammarSource {
    match grammar {
        Grammar::ObjectScript => GrammarSource {
            language: tree_sitter_objectscript_playground::LANGUAGE_OBJECTSCRIPT.into(),
            highlights_query: tree_sitter_objectscript_playground::STUDIO_HIGHLIGHTS_QUERY,
            injections_query: tree_sitter_objectscript_playground::INJECTIONS_QUERY,
        },
        Grammar::ObjectScriptRoutine => GrammarSource {
            language: tree_sitter_objectscript_routine::LANGUAGE_OBJECTSCRIPT_ROUTINE.into(),
            highlights_query: tree_sitter_objectscript_routine::STUDIO_HIGHLIGHTS_QUERY,
            injections_query: tree_sitter_objectscript_routine::INJECTIONS_QUERY,
        },
        Grammar::Sql => GrammarSource {
            language: SQL_LANGUAGE.into(),
            highlights_query: SQL_HIGHLIGHTS_QUERY,
            injections_query: "",
        },
        Grammar::Python => GrammarSource {
            language: tree_sitter_python::LANGUAGE.into(),
            highlights_query: tree_sitter_python::HIGHLIGHTS_QUERY,
            injections_query: "",
        },
        Grammar::Markdown => GrammarSource {
            language: MARKDOWN_LANGUAGE.into(),
            highlights_query: MARKDOWN_HIGHLIGHTS_QUERY,
            injections_query: MARKDOWN_INJECTIONS_QUERY,
        },
        // InterSystems MDX is OLAP query syntax; use SQL highlighting as a temporary fallback.
        Grammar::Mdx => GrammarSource {
            language: SQL_LANGUAGE.into(),
            highlights_query: SQL_HIGHLIGHTS_QUERY,
            injections_query: "",
        },
        Grammar::Xml => GrammarSource {
            language: XML_LANGUAGE.into(),
            highlights_query: XML_HIGHLIGHTS_QUERY,
            injections_query: XML_IMPLEMENTATION_INJECTIONS_QUERY,
        },
        Grammar::Json => GrammarSource {
            language: JSON_LANGUAGE.into(),
            highlights_query: JSON_HIGHLIGHTS_QUERY,
            injections_query: "",
        },
        Grammar::Yaml => GrammarSource {
            language: YAML_LANGUAGE.into(),
            highlights_query: YAML_HIGHLIGHTS_QUERY,
            injections_query: "",
        },
        Grammar::Css => GrammarSource {
            language: tree_sitter_css::LANGUAGE.into(),
            highlights_query: tree_sitter_css::HIGHLIGHTS_QUERY,
            injections_query: "",
        },
        Grammar::Html => GrammarSource {
            language: tree_sitter_html::LANGUAGE.into(),
            highlights_query: tree_sitter_html::HIGHLIGHTS_QUERY,
            injections_query: tree_sitter_html::INJECTIONS_QUERY,
        },
        Grammar::JavaScript => GrammarSource {
            language: tree_sitter_javascript::LANGUAGE.into(),
            highlights_query: tree_sitter_javascript::HIGHLIGHT_QUERY,
            injections_query: tree_sitter_javascript::INJECTIONS_QUERY,
        },
        Grammar::JsDoc => GrammarSource {
            language: tree_sitter_jsdoc::LANGUAGE.into(),
            highlights_query: tree_sitter_jsdoc::HIGHLIGHTS_QUERY,
            injections_query: "",
        },
        Grammar::ObjectScriptUdl => GrammarSource {
            language: tree_sitter_objectscript::LANGUAGE_OBJECTSCRIPT_UDL.into(),
            highlights_query: tree_sitter_objectscript::STUDIO_HIGHLIGHTS_QUERY,
            injections_query: tree_sitter_objectscript::INJECTIONS_QUERY,
        },
        Grammar::Regex => GrammarSource {
            language: tree_sitter_regex::LANGUAGE.into(),
            highlights_query: tree_sitter_regex::HIGHLIGHTS_QUERY,
            injections_query: "",
        },
        Grammar::Toml => GrammarSource {
            language: tree_sitter_toml_ng::LANGUAGE.into(),
            highlights_query: tree_sitter_toml_ng::HIGHLIGHTS_QUERY,
            injections_query: "",
        },
    }
}

fn builds_manual_injections(grammar: Grammar) -> bool {
    matches!(
        grammar,
        Grammar::ObjectScript | Grammar::ObjectScriptRoutine | Grammar::Xml
    )
}

fn register_capture_names(
    query: &Query,
    attr_ids_by_name: &mut HashMap<String, usize>,
    recognized: &mut Vec<String>,
) {
    for &capture_name in query.capture_names() {
        if attr_ids_by_name.contains_key(capture_name) {
            continue;
        }

        let id = recognized.len();
        let owned = capture_name.to_string();
        attr_ids_by_name.insert(owned.clone(), id);
        recognized.push(owned);
    }
}

fn env_flag_enabled(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn highlight_timing_logs_enabled_from_env() -> bool {
    match env::var(HIGHLIGHT_TIMING_ENV_VAR) {
        Ok(value) => env_flag_enabled(&value),
        Err(_) => false,
    }
}

fn format_duration(duration: Duration) -> String {
    format!("{:.1}ms", duration.as_secs_f64() * 1000.0)
}

fn log_highlight_success(
    grammar: Grammar,
    source_len: usize,
    span_count: usize,
    total: Duration,
    timing: HighlightTimingBreakdown,
) {
    eprintln!(
        "[syntax-color][highlight] {} {}B -> {} span(s) in {} (base {}, injection query {}, injection apply {}, injections {})",
        grammar.canonical_name(),
        source_len,
        span_count,
        format_duration(total),
        format_duration(timing.base),
        format_duration(timing.injection_query),
        format_duration(timing.injection_apply),
        timing.injection_count,
    );
}

fn log_highlight_failure(
    grammar: Grammar,
    source_len: usize,
    total: Duration,
    phase: &'static str,
    error: &HighlightError,
) {
    eprintln!(
        "[syntax-color][highlight] {} {}B failed during {} after {}: {}",
        grammar.canonical_name(),
        source_len,
        phase,
        format_duration(total),
        error,
    );
}

fn build_highlight_config(grammar: Grammar) -> Result<HighlightConfig, HighlightError> {
    let source = grammar_source(grammar);
    let mut parser = Parser::new();
    parser.set_language(&source.language)?;

    let highlight_query = Query::new(&source.language, source.highlights_query)?;
    let injection_query = if source.injections_query.is_empty() {
        None
    } else {
        Some(Query::new(&source.language, source.injections_query)?)
    };
    let (injection_content_capture_index, injection_language_capture_index) =
        if let Some(query) = injection_query.as_ref() {
            injection_capture_indices(query)
        } else {
            (None, None)
        };

    Ok(HighlightConfig {
        parser,
        highlight_query,
        injection_query,
        injection_content_capture_index,
        injection_language_capture_index,
    })
}

impl SpanHighlighter {
    /// Creates a highlighter configured for all supported grammars and injections.
    ///
    /// Queries and parsers are compiled once and then reused across highlight calls.
    ///
    /// # Errors
    ///
    /// Returns an error if any grammar query cannot be compiled or if parser
    /// language configuration fails.
    pub fn new() -> Result<Self, HighlightError> {
        let mut recognized = Vec::<String>::new();
        let mut attr_ids_by_name = HashMap::<String, usize>::new();
        let mut configs = HashMap::<Grammar, HighlightConfig>::new();

        for grammar in ALL_GRAMMARS {
            let config = build_highlight_config(grammar)?;
            register_capture_names(
                &config.highlight_query,
                &mut attr_ids_by_name,
                &mut recognized,
            );
            configs.insert(grammar, config);
        }

        let attrs = recognized
            .into_iter()
            .enumerate()
            .map(|(id, capture_name)| Attr { id, capture_name })
            .collect::<Vec<_>>();

        Ok(Self {
            configs,
            attrs,
            attr_ids_by_name,
            timing_logs_enabled: highlight_timing_logs_enabled_from_env(),
        })
    }

    /// Returns whether this highlighter emits per-highlight timing logs.
    #[must_use]
    pub fn timing_logs_enabled(&self) -> bool {
        self.timing_logs_enabled
    }

    /// Enables or disables per-highlight timing logs written to stderr.
    pub fn set_timing_logs_enabled(&mut self, enabled: bool) {
        self.timing_logs_enabled = enabled;
    }

    /// Returns a copy of this highlighter with timing logs enabled or disabled.
    #[must_use]
    pub fn with_timing_logs_enabled(mut self, enabled: bool) -> Self {
        self.set_timing_logs_enabled(enabled);
        self
    }

    /// Highlights a source buffer and returns capture attributes plus byte spans.
    ///
    /// When `grammar` is [`Grammar::ObjectScript`], language injections are resolved
    /// and applied to injected regions (for example embedded SQL blocks). When
    /// `grammar` is [`Grammar::Xml`], ObjectScript injections are applied to
    /// recognized XML embedded-code regions (for example `<Implementation>` bodies).
    ///
    /// # Errors
    ///
    /// Returns an error if Tree-sitter highlighting fails or if injection parsing
    /// cannot be completed.
    pub fn highlight(
        &mut self,
        source: &[u8],
        grammar: Grammar,
    ) -> Result<HighlightResult, HighlightError> {
        if self.timing_logs_enabled {
            return self.highlight_with_timing(source, grammar);
        }

        self.highlight_internal(source, grammar)
    }

    fn highlight_internal(
        &mut self,
        source: &[u8],
        grammar: Grammar,
    ) -> Result<HighlightResult, HighlightError> {
        let mut result = self.highlight_base(source, grammar)?;
        if builds_manual_injections(grammar) {
            let injections = self.find_injections(source, grammar)?;
            self.apply_injections(source, &mut result, injections)?;
        }
        Ok(result)
    }

    fn highlight_with_timing(
        &mut self,
        source: &[u8],
        grammar: Grammar,
    ) -> Result<HighlightResult, HighlightError> {
        let started_at = Instant::now();
        let mut timing = HighlightTimingBreakdown::default();

        let base_started_at = Instant::now();
        let mut result = match self.highlight_base(source, grammar) {
            Ok(result) => result,
            Err(error) => {
                log_highlight_failure(grammar, source.len(), started_at.elapsed(), "base", &error);
                return Err(error);
            }
        };
        timing.base = base_started_at.elapsed();

        if builds_manual_injections(grammar) {
            let injection_query_started_at = Instant::now();
            let injections = match self.find_injections(source, grammar) {
                Ok(injections) => injections,
                Err(error) => {
                    log_highlight_failure(
                        grammar,
                        source.len(),
                        started_at.elapsed(),
                        "injection query",
                        &error,
                    );
                    return Err(error);
                }
            };
            timing.injection_query = injection_query_started_at.elapsed();
            timing.injection_count = injections.len();

            if !injections.is_empty() {
                let injection_apply_started_at = Instant::now();
                if let Err(error) = self.apply_injections(source, &mut result, injections) {
                    log_highlight_failure(
                        grammar,
                        source.len(),
                        started_at.elapsed(),
                        "injection apply",
                        &error,
                    );
                    return Err(error);
                }
                timing.injection_apply = injection_apply_started_at.elapsed();
            }
        }

        let total = started_at.elapsed();
        log_highlight_success(grammar, source.len(), result.spans.len(), total, timing);
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
        grammar: Grammar,
    ) -> Result<HighlightResult, HighlightError> {
        let attrs = self.attrs.clone();
        let attr_ids_by_name = &self.attr_ids_by_name;
        let config = self
            .configs
            .get_mut(&grammar)
            .expect("all supported grammars should be preconfigured");
        let spans = Self::highlight_with_query_cursor(source, attr_ids_by_name, config)?;
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

    /// Finds and normalizes non-overlapping injection regions for a host grammar.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing or query execution for injection analysis fails.
    fn find_injections(
        &mut self,
        source: &[u8],
        grammar: Grammar,
    ) -> Result<Vec<InjectionRegion>, HighlightError> {
        let config = self
            .configs
            .get_mut(&grammar)
            .expect("all supported grammars should be preconfigured");
        let Some(query) = config.injection_query.as_ref() else {
            return Ok(Vec::new());
        };

        let tree = config
            .parser
            .parse(source, None)
            .ok_or(HighlightError::Parse)?;
        let mut cursor = QueryCursor::new();
        let mut injections = Vec::new();
        let mut matches = cursor.matches(query, tree.root_node(), source);
        while let Some(mat) = matches.next() {
            let Some(injection) = Self::injection_region_for_match(
                query,
                config.injection_content_capture_index,
                config.injection_language_capture_index,
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
        query: &Query,
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

    /// Highlights source using cached queries with playground-style precedence.
    ///
    /// Smaller overlapping ranges win over enclosing ranges. For equal ranges, more
    /// specific capture names win, and remaining ties fall back to later query patterns.
    fn highlight_with_query_cursor(
        source: &[u8],
        attr_ids_by_name: &HashMap<String, usize>,
        config: &mut HighlightConfig,
    ) -> Result<Vec<Span>, HighlightError> {
        let query = &config.highlight_query;
        let capture_names = query.capture_names();
        let tree = config
            .parser
            .parse(source, None)
            .ok_or(HighlightError::Parse)?;

        let mut candidates = Vec::<CaptureCandidate>::new();
        let mut order = 0usize;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(query, tree.root_node(), source);
        while {
            matches.advance();
            matches.get().is_some()
        } {
            let mat = matches.get().expect("match must exist after advance");
            for capture in mat.captures {
                let start_byte = capture.node.start_byte();
                let end_byte = capture.node.end_byte();
                if start_byte >= end_byte {
                    continue;
                }

                let capture_name = capture_names[capture.index as usize];
                let Some(&attr_id) = attr_ids_by_name.get(capture_name) else {
                    continue;
                };
                candidates.push(CaptureCandidate {
                    attr_id,
                    start_byte,
                    end_byte,
                    render_priority: capture_render_priority(capture_name),
                    specificity: capture_specificity(capture_name),
                    pattern_index: mat.pattern_index,
                    order,
                });
                order += 1;
            }
        }

        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        let mut events = Vec::<(usize, bool, usize)>::with_capacity(candidates.len() * 2);
        for (idx, candidate) in candidates.iter().enumerate() {
            // End events are processed before start events at the same byte offset.
            events.push((candidate.start_byte, true, idx));
            events.push((candidate.end_byte, false, idx));
        }
        events.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        let mut spans = Vec::<Span>::new();
        let mut active = Vec::<usize>::new();
        let mut cursor_byte = events[0].0;
        let mut i = 0usize;

        while i < events.len() {
            let offset = events[i].0;

            if cursor_byte < offset {
                if let Some(best_idx) = active.iter().copied().max_by_key(|idx| {
                    let candidate = &candidates[*idx];
                    (
                        candidate.render_priority,
                        Reverse(candidate.end_byte - candidate.start_byte),
                        candidate.specificity,
                        candidate.pattern_index,
                        candidate.order,
                    )
                }) {
                    push_merged(
                        &mut spans,
                        Span {
                            attr_id: candidates[best_idx].attr_id,
                            start_byte: cursor_byte,
                            end_byte: offset,
                        },
                    );
                }
            }

            while i < events.len() && events[i].0 == offset {
                let (_, is_start, idx) = events[i];
                if is_start {
                    active.push(idx);
                } else if let Some(pos) = active.iter().position(|active_idx| *active_idx == idx) {
                    active.swap_remove(pos);
                }
                i += 1;
            }
            cursor_byte = offset;
        }

        Ok(normalize_spans(spans))
    }
}

#[cfg(test)]
mod tests {
    use super::env_flag_enabled;

    #[test]
    fn parses_truthy_timing_log_values() {
        assert!(env_flag_enabled("1"));
        assert!(env_flag_enabled("true"));
        assert!(env_flag_enabled(" TRUE "));
        assert!(env_flag_enabled("yes"));
        assert!(env_flag_enabled("on"));
    }

    #[test]
    fn rejects_falsey_timing_log_values() {
        assert!(!env_flag_enabled("0"));
        assert!(!env_flag_enabled("false"));
        assert!(!env_flag_enabled("off"));
        assert!(!env_flag_enabled(""));
    }
}
