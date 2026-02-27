# Data Structure: HighlightResult

## Overview

- `HighlightResult` is the parser-stage output that carries semantic attributes plus highlighted byte ranges (`crates/highlight-spans/src/lib.rs:32`).
- Primary consumers are downstream renderers such as `render-ansi` that map attr IDs to styles (`crates/render-ansi/src/lib.rs:33`, `crates/render-ansi/src/lib.rs:37`).

## Scope

- In scope: `highlight-spans` creation path and downstream use in `render-ansi` (`crates/highlight-spans/src/lib.rs:83`, `crates/render-ansi/src/lib.rs:32`).
- Out of scope: terminal transport/output behavior after ANSI string generation (`crates/render-ansi/src/lib.rs:53`).

## Canonical Definition

- Canonical struct definition: `pub struct HighlightResult { pub attrs: Vec<Attr>, pub spans: Vec<Span> }` (`crates/highlight-spans/src/lib.rs:32`).
- `Attr` and `Span` are defined in the same module and are part of the same data contract (`crates/highlight-spans/src/lib.rs:12`, `crates/highlight-spans/src/lib.rs:25`).

## Fields and Types

- `attrs: Vec<Attr>` where `Attr` has `id: usize` and `capture_name: String` (`crates/highlight-spans/src/lib.rs:13`, `crates/highlight-spans/src/lib.rs:14`, `crates/highlight-spans/src/lib.rs:33`).
- `spans: Vec<Span>` where `Span` has `attr_id`, `start_byte`, and `end_byte` as `usize` (`crates/highlight-spans/src/lib.rs:26`, `crates/highlight-spans/src/lib.rs:27`, `crates/highlight-spans/src/lib.rs:28`, `crates/highlight-spans/src/lib.rs:34`).
- No default values are defined; the structure is constructed by `highlight` at runtime (`crates/highlight-spans/src/lib.rs:83`, `crates/highlight-spans/src/lib.rs:129`).

## Invariants

- `push_merged` drops zero-length spans (`start_byte >= end_byte`) (`crates/highlight-spans/src/lib.rs:165`).
- Adjacent spans with same `attr_id` are merged to reduce fragmentation (`crates/highlight-spans/src/lib.rs:170`).
- Downstream renderer expects span order to be sorted and non-overlapping; invalid sequences are rejected (`crates/render-ansi/src/lib.rs:217`, `crates/render-ansi/src/lib.rs:227`).

## Ownership and Responsibilities

- `SpanHighlighter` owns creation of `HighlightResult` (`crates/highlight-spans/src/lib.rs:45`, `crates/highlight-spans/src/lib.rs:83`).
- Renderer modules own validation and style resolution based on this structure (`crates/render-ansi/src/lib.rs:32`, `crates/render-ansi/src/lib.rs:54`).

## Lifecycle

- Creation path: `SpanHighlighter::highlight` builds `attrs`, walks `HighlightEvent`s, then returns `HighlightResult` (`crates/highlight-spans/src/lib.rs:94`, `crates/highlight-spans/src/lib.rs:108`, `crates/highlight-spans/src/lib.rs:129`).
- Update path: no in-place mutation API exists after construction; consumers read and transform into new collections (`crates/highlight-spans/src/lib.rs:32`, `crates/render-ansi/src/lib.rs:36`).
- Deletion/retention path: lifetime is process memory and caller-owned; no persistence logic exists (`crates/highlight-spans/src/lib.rs:83`).

## Update and Maintenance

- Primary update flow is grammar/query evolution affecting emitted capture names and spans (`crates/highlight-spans/src/lib.rs:55`, `crates/highlight-spans/src/lib.rs:57`).
- Background jobs or batch processes: Not applicable.
- Migration strategy: contract remains byte-range plus attr-table; downstreams should keep attr ID lookup and validation logic (`crates/highlight-spans/src/lib.rs:32`, `crates/render-ansi/src/lib.rs:38`).

## Storage and Access

- Stored in-memory as vectors during request/operation scope (`crates/highlight-spans/src/lib.rs:33`, `crates/highlight-spans/src/lib.rs:34`).
- Access is index-based (`attrs[attr_id]`) in downstream resolution (`crates/render-ansi/src/lib.rs:38`).

## APIs and Interfaces

- Produced by `SpanHighlighter::highlight` and `SpanHighlighter::highlight_lines` (`crates/highlight-spans/src/lib.rs:83`, `crates/highlight-spans/src/lib.rs:132`).
- Consumed by `resolve_styled_spans` to derive `StyledSpan` (`crates/render-ansi/src/lib.rs:32`).

## Usage Examples

- Example flow: `highlight(...) -> HighlightResult -> resolve_styled_spans(...)` (`crates/render-ansi/src/lib.rs:133`, `crates/render-ansi/src/lib.rs:134`).
- Numeric literal coverage test demonstrates capture and span lookup by `attr_id` (`crates/highlight-spans/src/lib.rs:199`, `crates/highlight-spans/src/lib.rs:207`).

## Pitfalls and Edge Cases

- Invalid `attr_id` in spans causes renderer error (`InvalidAttrId`) (`crates/render-ansi/src/lib.rs:29`, `crates/render-ansi/src/lib.rs:39`).
- Overlapping spans are rejected during rendering (`crates/render-ansi/src/lib.rs:227`).
- Byte ranges are byte-based, so consumers must keep source bytes aligned with original highlighted input (`crates/highlight-spans/src/lib.rs:85`, `crates/render-ansi/src/lib.rs:64`).

## Observability

Not applicable.

No logging/metrics are emitted by this data structure in current code (`crates/highlight-spans/src/lib.rs:83`, `crates/render-ansi/src/lib.rs:53`).

## Security and Privacy

- Structure contains syntax metadata and byte offsets, not user identity fields (`crates/highlight-spans/src/lib.rs:13`, `crates/highlight-spans/src/lib.rs:27`).
- Access control/compliance handling is out of scope for this library data model (`crates/highlight-spans/src/lib.rs:83`, `crates/render-ansi/src/lib.rs:53`).

## Assumptions

- Consumers treat `HighlightResult` as immutable once returned.
- Span ordering emitted by `highlight-spans` is expected by downstream renderers.
