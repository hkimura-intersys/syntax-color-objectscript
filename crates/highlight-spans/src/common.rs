use crate::highlight_structures::*;
use std::collections::HashMap;

/// Computes a simple specificity score for a capture name.
///
/// More dotted segments means more specific (for example
/// `string.special.key` > `string.special` > `string`).
pub fn capture_specificity(capture_name: &str) -> usize {
    capture_name
        .split('.')
        .filter(|segment| !segment.is_empty())
        .count()
}

/// Returns a precedence bucket for choosing the visible capture.
///
/// Tree-sitter uses auxiliary captures like `spell` and `nospell` to annotate
/// spell-check behavior. They should not override semantic syntax coloring when
/// both are active on the same bytes.
pub fn capture_render_priority(capture_name: &str) -> usize {
    match capture_name.trim_start_matches('@') {
        "spell" | "nospell" => 0,
        _ => 1,
    }
}

/// Locates `injection.content` and `injection.language` captures in a query.
pub fn injection_capture_indices(query: &tree_sitter::Query) -> (Option<u32>, Option<u32>) {
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

/// Pushes a span into `spans`, merging with the previous span when adjacent and
/// sharing the same attribute id.
pub fn push_merged(spans: &mut Vec<Span>, next: Span) {
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
pub fn remap_attr_ids(
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
pub fn exclude_ranges(spans: &[Span], ranges: &[(usize, usize)]) -> Vec<Span> {
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
pub fn normalize_spans(mut spans: Vec<Span>) -> Vec<Span> {
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

