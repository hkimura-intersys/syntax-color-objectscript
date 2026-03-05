# Data Structure: StyledSpan

## Overview

- `StyledSpan` is the render-stage structure that pairs byte ranges with an optional resolved style (`crates/render-ansi/src/lib.rs:16`).
- Primary consumers are `render_ansi`, `render_ansi_lines`, and incremental projection in `IncrementalRenderer::render_patch` (`crates/render-ansi/src/lib.rs:89`, `crates/render-ansi/src/lib.rs:264`, `crates/render-ansi/src/lib.rs:295`).

## Scope

- In scope: style resolution, span validation, ANSI rendering, and incremental VT patch generation (`crates/render-ansi/src/lib.rs:238`, `crates/render-ansi/src/lib.rs:497`, `crates/render-ansi/src/lib.rs:662`).
- Out of scope: syntax extraction and theme authoring (`crates/highlight-spans/src/lib.rs:228`, `crates/theme-engine/src/lib.rs:136`).

## Canonical Definition

- Canonical struct: `StyledSpan { start_byte, end_byte, style: Option<Style> }` (`crates/render-ansi/src/lib.rs:16`, `crates/render-ansi/src/lib.rs:19`).
- `Style` is imported from `theme-engine` (`crates/render-ansi/src/lib.rs:5`).

## Fields and Types

- `start_byte: usize` (`crates/render-ansi/src/lib.rs:17`).
- `end_byte: usize` (`crates/render-ansi/src/lib.rs:18`).
- `style: Option<Style>` where `None` means plain-text emission (`crates/render-ansi/src/lib.rs:19`, `crates/render-ansi/src/lib.rs:596`).

## Invariants

- Renderer requires `start_byte <= end_byte <= source_len` (`crates/render-ansi/src/lib.rs:665`).
- Renderer requires sorted, non-overlapping spans (`crates/render-ansi/src/lib.rs:672`).
- Empty text segments are ignored during append (`crates/render-ansi/src/lib.rs:585`).

## Ownership and Responsibilities

- `resolve_styled_spans` owns conversion from `HighlightResult` attrs/spans to `StyledSpan` values (`crates/render-ansi/src/lib.rs:238`).
- Render paths consume validated `StyledSpan` data:
  - full-buffer: `render_ansi` (`crates/render-ansi/src/lib.rs:264`)
  - per-line: `render_ansi_lines` (`crates/render-ansi/src/lib.rs:295`)
  - incremental: `IncrementalRenderer::render_patch` (`crates/render-ansi/src/lib.rs:89`)

## Lifecycle

- Creation path: `highlight_to_ansi_with_highlighter` highlights then resolves styled spans (`crates/render-ansi/src/lib.rs:357`, `crates/render-ansi/src/lib.rs:363`, `crates/render-ansi/src/lib.rs:364`).
- Update path: spans are immutable render inputs; incremental history is stored separately as cached `StyledCell` lines (`crates/render-ansi/src/lib.rs:35`, `crates/render-ansi/src/lib.rs:95`, `crates/render-ansi/src/lib.rs:102`).
- Retention path: `Vec<StyledSpan>` is transient per call; incremental caches persist across calls until `clear_state` (`crates/render-ansi/src/lib.rs:62`).

## Incremental Rendering Notes

- Incremental projection is grapheme-based and tracks display width per cell (`crates/render-ansi/src/lib.rs:421`, `crates/render-ansi/src/lib.rs:475`).
- Diff cursor columns are display-width based, not raw byte based (`crates/render-ansi/src/lib.rs:497`, `crates/render-ansi/src/lib.rs:516`, `crates/render-ansi/src/lib.rs:531`).
- Renderer origin offsets (`row`, `col`) are configurable for prompt-aware patch positioning (`crates/render-ansi/src/lib.rs:70`).
- Tabs are expanded using an 8-column tab stop in incremental width calculations (`crates/render-ansi/src/lib.rs:13`, `crates/render-ansi/src/lib.rs:483`).

## APIs and Interfaces

- Produced by `resolve_styled_spans` (`crates/render-ansi/src/lib.rs:238`).
- Consumed by `render_ansi`, `render_ansi_lines`, `render_patch`, and `highlight_to_patch` (`crates/render-ansi/src/lib.rs:89`, `crates/render-ansi/src/lib.rs:111`, `crates/render-ansi/src/lib.rs:264`, `crates/render-ansi/src/lib.rs:295`).
- For multiplexed terminals, keep session-scoped incremental state in your host (for example `HashMap<String, IncrementalRenderer>`) and call `IncrementalRenderer::highlight_to_patch` per terminal ID (`crates/render-ansi/src/lib.rs:167`).

## Usage Examples

- Full pipeline: `highlight_to_ansi` (`crates/render-ansi/src/lib.rs:343`).
- Incremental with origin: `set_origin` + `highlight_to_patch` (`crates/render-ansi/src/lib.rs:70`, `crates/render-ansi/src/lib.rs:111`).
- Tests validate display-width and tab behavior (`crates/render-ansi/src/lib.rs:816`, `crates/render-ansi/src/lib.rs:830`).

## Pitfalls and Edge Cases

- Invalid `attr_id` causes immediate error during span resolution (`crates/render-ansi/src/lib.rs:244`).
- Overlap/out-of-bounds violations fail before rendering (`crates/render-ansi/src/lib.rs:665`, `crates/render-ansi/src/lib.rs:672`).
- Incremental display-width handling is independent from host terminal soft-wrap policies; host and renderer viewport/origin must agree.

## Security and Privacy

- Structure carries offsets and style metadata only (`crates/render-ansi/src/lib.rs:17`, `crates/render-ansi/src/lib.rs:19`).
- Range checks are enforced before any slicing (`crates/render-ansi/src/lib.rs:265`, `crates/render-ansi/src/lib.rs:662`).

## Assumptions

- Source bytes are unchanged between highlight and render.
- Upstream producers provide sorted/non-overlapping spans.
- Host applications provide correct viewport/origin context for incremental rendering.
