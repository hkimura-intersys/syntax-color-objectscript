# arc42: syntax-color-objectscript (Repo Scope)

## 1. Introduction and Goals

This document covers the full Rust workspace that provides ObjectScript highlighting data and ANSI rendering adapters (`Cargo.toml:1`, `Cargo.toml:2`).
Primary audience: maintainers of this workspace and integrators embedding the crates in CLI/TUI tools (`crates/render-ansi/src/lib.rs:118`, `crates/highlight-spans/src/lib.rs:45`).
Goal: keep parser output, theme lookup, and rendering concerns independently evolvable (`crates/render-ansi/Cargo.toml:9`, `crates/render-ansi/Cargo.toml:10`, `crates/theme-engine/Cargo.toml:1`).

## 2. Constraints

Technical constraints:
- Rust 2021 workspace with resolver v2 (`Cargo.toml:3`, `Cargo.toml:6`).
- Highlight extraction is constrained by Tree-sitter APIs and the `tree-sitter-objectscript` grammar package (`crates/highlight-spans/Cargo.toml:10`, `crates/highlight-spans/Cargo.toml:12`).
- Theme ingestion is constrained to JSON/TOML serde parsing and built-in JSON assets (`crates/theme-engine/src/lib.rs:134`, `crates/theme-engine/src/lib.rs:139`, `crates/theme-engine/src/lib.rs:45`).

Operational constraints:
- This repository exports libraries only; runtime environment is controlled by downstream applications (`crates/highlight-spans/Cargo.toml:1`, `crates/render-ansi/Cargo.toml:1`, `crates/theme-engine/Cargo.toml:1`).

## 3. Context and Scope

External actors/systems:
- Host applications call public crate APIs to highlight text and/or render ANSI (`crates/render-ansi/src/lib.rs:118`, `crates/highlight-spans/src/lib.rs:83`).
- Tree-sitter grammar/query assets are external dependencies consumed by `highlight-spans` (`crates/highlight-spans/src/lib.rs:55`, `crates/highlight-spans/src/lib.rs:57`).

In scope:
- `highlight-spans`, `theme-engine`, and `render-ansi` crate behavior and interfaces (`Cargo.toml:2`).

Out of scope:
- Non-ANSI renderer implementations and host app I/O orchestration (not implemented in this workspace; `crates/render-ansi/src/lib.rs:53`).

## 4. Solution Strategy

The workspace strategy is compositional:
- Produce semantic capture spans from source (`crates/highlight-spans/src/lib.rs:83`, `crates/highlight-spans/src/lib.rs:129`).
- Resolve capture names into styles with normalization and hierarchical fallback (`crates/theme-engine/src/lib.rs:117`, `crates/theme-engine/src/lib.rs:125`, `crates/theme-engine/src/lib.rs:193`).
- Render styled ranges into ANSI-escaped output (`crates/render-ansi/src/lib.rs:53`, `crates/render-ansi/src/lib.rs:168`).

This fits the constraints because parser/grammar changes, theme changes, and output adapter changes remain localized to separate crates (`Cargo.toml:2`).

## 5. Building Block View

Main building blocks:
- `highlight-spans`: `SpanHighlighter` returns `HighlightResult { attrs, spans }` (`crates/highlight-spans/src/lib.rs:45`, `crates/highlight-spans/src/lib.rs:32`).
- `theme-engine`: `Theme` maps normalized capture keys to `Style` and supports built-ins (`crates/theme-engine/src/lib.rs:82`, `crates/theme-engine/src/lib.rs:117`, `crates/theme-engine/src/lib.rs:144`).
- `render-ansi`: converts `HighlightResult` + `Theme` into ANSI `String` or per-line `Vec<String>` (`crates/render-ansi/src/lib.rs:32`, `crates/render-ansi/src/lib.rs:53`, `crates/render-ansi/src/lib.rs:77`).

Key interactions:
- `render-ansi` depends on `highlight-spans` and `theme-engine` via path dependencies (`crates/render-ansi/Cargo.toml:9`, `crates/render-ansi/Cargo.toml:10`).

## 6. Runtime View

Critical flow:
1. `highlight_to_ansi` creates a `SpanHighlighter` and requests spans (`crates/render-ansi/src/lib.rs:123`, `crates/render-ansi/src/lib.rs:133`).
2. `resolve_styled_spans` converts span attr IDs to optional `Style` using `Theme::resolve` (`crates/render-ansi/src/lib.rs:32`, `crates/render-ansi/src/lib.rs:47`).
3. `render_ansi` validates span order/bounds, then emits ANSI open/reset codes around styled byte segments (`crates/render-ansi/src/lib.rs:54`, `crates/render-ansi/src/lib.rs:217`, `crates/render-ansi/src/lib.rs:171`).

Side-effect points:
- Functions are pure in-memory transforms except allocation/formatting; no filesystem/network activity is present in crate APIs (`crates/render-ansi/src/lib.rs:56`, `crates/theme-engine/src/lib.rs:134`).

## 7. Deployment View

Not applicable.

Reason: workspace contains reusable libraries and tests, without in-repo daemon/service deployment descriptors (`Cargo.toml:1`, `crates/highlight-spans/Cargo.toml:1`).

## 8. Cross-Cutting Concepts

- Error handling uses typed enums via `thiserror` in both highlight and render stages (`crates/highlight-spans/src/lib.rs:37`, `crates/render-ansi/src/lib.rs:14`, `crates/theme-engine/src/lib.rs:164`).
- Capture naming is normalized (`@` stripping, lowercase) to reduce style key mismatch (`crates/theme-engine/src/lib.rs:193`).
- Span safety is enforced before rendering through explicit bounds and overlap checks (`crates/render-ansi/src/lib.rs:217`, `crates/render-ansi/src/lib.rs:227`).

## 9. Architectural Decisions

- Decision: use attr-table + span tuples instead of per-character styles to keep highlight output compact (`crates/highlight-spans/src/lib.rs:32`, `crates/highlight-spans/src/lib.rs:25`).
- Decision: dotted fallback in theme resolution (`comment.documentation -> comment -> normal`) to reduce required theme verbosity (`crates/theme-engine/src/lib.rs:125`, `crates/theme-engine/src/lib.rs:131`).
- Decision: provide both whole-buffer and line-oriented rendering APIs for terminal integration flexibility (`crates/render-ansi/src/lib.rs:53`, `crates/render-ansi/src/lib.rs:77`).

## 10. Quality Requirements

Reliability:
- Highlight, theme, and render flows each have unit tests covering critical behavior (`crates/highlight-spans/src/lib.rs:183`, `crates/theme-engine/src/lib.rs:206`, `crates/render-ansi/src/lib.rs:247`).
- Renderer rejects invalid span ordering and out-of-bounds ranges (`crates/render-ansi/src/lib.rs:217`).

Maintainability:
- Workspace-level crate split enforces module boundaries (`Cargo.toml:2`).

## 11. Risks and Technical Debt

- ANSI renderer always appends reset after styled segments, which may increase escape-sequence volume for dense highlighting (`crates/render-ansi/src/lib.rs:171`).
- Span validation occurs at render time, so upstream producers outside this workspace must still maintain sorted/non-overlapping contracts (`crates/render-ansi/src/lib.rs:217`).
- No non-ANSI adapter crate is included yet, so downstreams must build alternatives independently (`crates/render-ansi/src/lib.rs:53`).

## 12. Glossary

- Attr: mapping from numeric `id` to capture name (`crates/highlight-spans/src/lib.rs:12`).
- Span: byte range tagged with `attr_id` (`crates/highlight-spans/src/lib.rs:25`).
- Theme: normalized capture-to-style lookup map (`crates/theme-engine/src/lib.rs:82`).
- StyledSpan: render-time span with resolved optional style (`crates/render-ansi/src/lib.rs:8`).

### Critical Data Structures

- `HighlightResult` (`crates/highlight-spans/src/lib.rs:32`)
- `Theme` (`crates/theme-engine/src/lib.rs:82`)
- `StyledSpan` (`crates/render-ansi/src/lib.rs:8`)

## Assumptions

- Downstream integrations handle terminal mode/state outside returned ANSI strings.
- Current architecture intentionally targets library reuse, not end-user CLI packaging.

## Open Questions

- Should the workspace include a non-ANSI renderer crate to standardize host integrations?
- Should span validation move earlier in the pipeline for clearer fault isolation?

## Evidence

- `Cargo.toml:1`
- `Cargo.toml:2`
- `Cargo.toml:3`
- `Cargo.toml:6`
- `crates/highlight-spans/Cargo.toml:10`
- `crates/highlight-spans/Cargo.toml:12`
- `crates/highlight-spans/src/lib.rs:12`
- `crates/highlight-spans/src/lib.rs:25`
- `crates/highlight-spans/src/lib.rs:32`
- `crates/highlight-spans/src/lib.rs:45`
- `crates/highlight-spans/src/lib.rs:83`
- `crates/highlight-spans/src/lib.rs:129`
- `crates/highlight-spans/src/lib.rs:183`
- `crates/theme-engine/src/lib.rs:45`
- `crates/theme-engine/src/lib.rs:82`
- `crates/theme-engine/src/lib.rs:117`
- `crates/theme-engine/src/lib.rs:125`
- `crates/theme-engine/src/lib.rs:131`
- `crates/theme-engine/src/lib.rs:134`
- `crates/theme-engine/src/lib.rs:139`
- `crates/theme-engine/src/lib.rs:144`
- `crates/theme-engine/src/lib.rs:164`
- `crates/theme-engine/src/lib.rs:193`
- `crates/theme-engine/src/lib.rs:206`
- `crates/render-ansi/Cargo.toml:9`
- `crates/render-ansi/Cargo.toml:10`
- `crates/render-ansi/src/lib.rs:8`
- `crates/render-ansi/src/lib.rs:14`
- `crates/render-ansi/src/lib.rs:32`
- `crates/render-ansi/src/lib.rs:47`
- `crates/render-ansi/src/lib.rs:53`
- `crates/render-ansi/src/lib.rs:54`
- `crates/render-ansi/src/lib.rs:77`
- `crates/render-ansi/src/lib.rs:118`
- `crates/render-ansi/src/lib.rs:123`
- `crates/render-ansi/src/lib.rs:133`
- `crates/render-ansi/src/lib.rs:168`
- `crates/render-ansi/src/lib.rs:171`
- `crates/render-ansi/src/lib.rs:217`
- `crates/render-ansi/src/lib.rs:227`
- `crates/render-ansi/src/lib.rs:247`
