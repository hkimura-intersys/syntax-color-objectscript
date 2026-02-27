# Data Structure: StyledSpan

## Overview

- `StyledSpan` is the render-stage structure that pairs byte ranges with an optional resolved style (`crates/render-ansi/src/lib.rs:8`).
- Primary consumers are `render_ansi` and `render_ansi_lines` (`crates/render-ansi/src/lib.rs:53`, `crates/render-ansi/src/lib.rs:77`).

## Scope

- In scope: `render-ansi` transformation and validation pipeline (`crates/render-ansi/src/lib.rs:32`, `crates/render-ansi/src/lib.rs:217`).
- Out of scope: syntax extraction and theme-map internals (`crates/highlight-spans/src/lib.rs:83`, `crates/theme-engine/src/lib.rs:82`).

## Canonical Definition

- Canonical struct: `StyledSpan { start_byte, end_byte, style: Option<Style> }` (`crates/render-ansi/src/lib.rs:8`, `crates/render-ansi/src/lib.rs:11`).
- `Style` is imported from `theme-engine` (`crates/render-ansi/src/lib.rs:2`).

## Fields and Types

- `start_byte: usize` (`crates/render-ansi/src/lib.rs:9`).
- `end_byte: usize` (`crates/render-ansi/src/lib.rs:10`).
- `style: Option<Style>` where `None` means plain-text emission (`crates/render-ansi/src/lib.rs:11`, `crates/render-ansi/src/lib.rs:178`).
- No default values or constructor helpers are defined; values are assigned directly by transformation code (`crates/render-ansi/src/lib.rs:44`).

## Invariants

- Renderer requires `start_byte <= end_byte <= source_len` (`crates/render-ansi/src/lib.rs:220`).
- Renderer requires sorted, non-overlapping spans (`crates/render-ansi/src/lib.rs:227`).
- Empty text segments are ignored during append (`crates/render-ansi/src/lib.rs:164`).

## Ownership and Responsibilities

- `resolve_styled_spans` owns creation by joining `HighlightResult` with `Theme` lookup (`crates/render-ansi/src/lib.rs:32`, `crates/render-ansi/src/lib.rs:47`).
- Rendering functions own consumption and span validation (`crates/render-ansi/src/lib.rs:54`, `crates/render-ansi/src/lib.rs:78`).

## Lifecycle

- Creation path: built per request from highlight result and theme (`crates/render-ansi/src/lib.rs:36`, `crates/render-ansi/src/lib.rs:44`).
- Update path: no mutation helpers are exposed; pipeline treats collection as immutable input after creation (`crates/render-ansi/src/lib.rs:53`).
- Deletion/retention path: transient in-memory vectors dropped after rendering call returns (`crates/render-ansi/src/lib.rs:35`, `crates/render-ansi/src/lib.rs:53`).

## Update and Maintenance

- Primary update flow tracks changes in `Theme::resolve` semantics and highlight attr IDs (`crates/theme-engine/src/lib.rs:117`, `crates/render-ansi/src/lib.rs:38`).
- Background jobs or batch processes: Not applicable.
- Migration strategy: keep `StyledSpan` byte-range contract aligned with `HighlightResult` span contract (`crates/highlight-spans/src/lib.rs:25`, `crates/render-ansi/src/lib.rs:9`).

## Storage and Access

- Stored in-memory as `Vec<StyledSpan>` for full-buffer and line rendering paths (`crates/render-ansi/src/lib.rs:35`, `crates/render-ansi/src/lib.rs:159`).
- Access pattern is sequential scan with cursor tracking for efficient pass-through rendering (`crates/render-ansi/src/lib.rs:57`, `crates/render-ansi/src/lib.rs:91`).

## APIs and Interfaces

- Produced by `resolve_styled_spans` (`crates/render-ansi/src/lib.rs:32`).
- Consumed by `render_ansi` and `render_ansi_lines` (`crates/render-ansi/src/lib.rs:53`, `crates/render-ansi/src/lib.rs:77`).
- Also produced/consumed indirectly through `highlight_to_ansi*` helper APIs (`crates/render-ansi/src/lib.rs:127`, `crates/render-ansi/src/lib.rs:153`).

## Usage Examples

- `highlight_to_ansi_with_highlighter` creates highlight result, resolves styled spans, then renders (`crates/render-ansi/src/lib.rs:133`, `crates/render-ansi/src/lib.rs:134`, `crates/render-ansi/src/lib.rs:135`).
- Unit tests build `StyledSpan` literals directly to assert SGR behavior and overlap rejection (`crates/render-ansi/src/lib.rs:250`, `crates/render-ansi/src/lib.rs:283`).

## Pitfalls and Edge Cases

- Invalid `attr_id` during creation causes immediate error and no partial output (`crates/render-ansi/src/lib.rs:38`, `crates/render-ansi/src/lib.rs:39`).
- Overlap or out-of-bounds violations fail rendering (`crates/render-ansi/src/lib.rs:220`, `crates/render-ansi/src/lib.rs:228`).
- Multi-line spans are clipped per line in `render_ansi_lines`; this is validated by tests (`crates/render-ansi/src/lib.rs:98`, `crates/render-ansi/src/lib.rs:264`).

## Observability

Not applicable.

No logging or metric hooks are associated with this structure in current implementation (`crates/render-ansi/src/lib.rs:53`).

## Security and Privacy

- Structure contains offsets and style metadata only (`crates/render-ansi/src/lib.rs:9`, `crates/render-ansi/src/lib.rs:11`).
- Safety against malformed ranges is enforced before slice access (`crates/render-ansi/src/lib.rs:54`, `crates/render-ansi/src/lib.rs:220`).

## Assumptions

- Input source bytes remain unchanged between span creation and render call.
- Styled spans are produced from trusted in-process transformations rather than external unvalidated input.
