# Edge-Case Test Plan: syntax-color-objectscript (Repo Scope)

Execution Status: Pending user approval.

## 1. Scope and Objectives

- Scope mode: `repo`
- In scope: crate-level APIs in `highlight-spans`, `theme-engine`, and `render-ansi`.
- Out of scope: host application integration, terminal emulator behavior, and non-ANSI render backends.
- Confidence target and release context: high confidence on parser-to-render correctness and error handling before publishing reusable library updates.

## 2. System Under Test

- Key components/symbols covered:
  - `SpanHighlighter::highlight`, `SpanHighlighter::highlight_lines`
  - `Theme::resolve`, `load_theme`
  - `resolve_styled_spans`, `render_ansi`, `render_ansi_lines`, `highlight_to_ansi`
- External interfaces and dependencies:
  - `tree-sitter` and `tree-sitter-objectscript` highlight query dependencies
  - JSON/TOML parsing via serde/toml
- Important stateful boundaries:
  - Byte-range span ordering and bounds
  - Capture-name normalization and fallback chain

## 3. Risk Model

- Quality attributes: correctness, reliability, maintainability, performance.
- Top failure modes:
  - invalid span ranges causing panics or incorrect rendering,
  - capture-name mismatch causing style fallback errors,
  - large multiline inputs amplifying allocation overhead.
- Risk ranking method used: impact (1-5) x likelihood (1-5), mapped to `P0`/`P1`/`P2`.

## 4. Test Strategy

- Chosen test levels: primarily `unit` with selected `integration` tests through orchestration APIs.
- Fixture/mocking approach: static inline source/theme fixtures; no network or filesystem mocks required.
- Deterministic vs non-deterministic handling: deterministic assertions on string output, style resolution, and error variants.

## 5. Scenario Matrix

| ID | Priority | Level | Area | Scenario | Expected Result | Status | Owner Test File |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EC-HL-001 | P0 | unit | Highlight | Numeric literal capture emits `number` span | At least one span maps to `number` and source slice `42` | existing | `crates/highlight-spans/src/lib.rs` |
| EC-HL-002 | P1 | unit | Highlight | Zero-length source events are ignored by span merger | No span with `start_byte >= end_byte` is emitted | new | `crates/highlight-spans/src/lib.rs` |
| EC-HL-003 | P1 | integration | Highlight | Single grammar mode (`ObjectScript`) produces valid results | Highlight returns `Ok` and spans are internally consistent | new | `crates/highlight-spans/src/lib.rs` |
| EC-TH-001 | P0 | unit | Theme | Dotted fallback (`comment.documentation -> comment`) | `resolve` returns parent style when leaf missing | existing | `crates/theme-engine/src/lib.rs` |
| EC-TH-002 | P0 | unit | Theme | Unknown built-in name is rejected | `ThemeError::UnknownBuiltinTheme` | existing | `crates/theme-engine/src/lib.rs` |
| EC-TH-003 | P1 | unit | Theme | Missing `normal` style and unknown capture | `resolve` returns `None` | new | `crates/theme-engine/src/lib.rs` |
| EC-RN-001 | P0 | unit | Render | Overlapping spans | `RenderError::OverlappingSpans` | existing | `crates/render-ansi/src/lib.rs` |
| EC-RN-002 | P0 | unit | Render | Multi-line span clipping | Per-line output clips span bounds without panic | existing | `crates/render-ansi/src/lib.rs` |
| EC-RN-003 | P0 | unit | Render | Out-of-bounds span range | `RenderError::SpanOutOfBounds` | new | `crates/render-ansi/src/lib.rs` |
| EC-RN-004 | P1 | integration | Render | Invalid UTF-8 bytes in source segment | Rendering succeeds with lossy UTF-8 conversion | new | `crates/render-ansi/src/lib.rs` |

## 6. Existing Coverage Map

- `EC-HL-001`: covered by `highlights_numeric_literal_as_number`.
- `EC-TH-001`: covered by `resolves_dot_fallback_then_normal`.
- `EC-TH-002`: covered by `rejects_unknown_built_in_theme_name`.
- `EC-RN-001`: covered by `rejects_overlapping_spans`.
- `EC-RN-002`: covered by `renders_per_line_output_for_multiline_span`.

Blind spots:
- Explicit out-of-bounds rendering rejection test (`EC-RN-003`) is missing.
- Single-grammar smoke coverage needs direct assertion (`EC-HL-003`).
- Missing-`normal` fallback behavior lacks direct assertion (`EC-TH-003`).

## 7. Implementation Plan

1. Add `EC-RN-003` out-of-bounds span test in `render-ansi`.
2. Add `EC-HL-003` single-grammar smoke test in `highlight-spans`.
3. Add `EC-TH-003` no-`normal` fallback test in `theme-engine`.
4. Add `EC-HL-002` and `EC-RN-004` for edge handling in merge/render paths.

Dependencies and blocking prerequisites:
- None beyond current workspace dependencies.

Expected effort/risk notes:
- Most additions are low-risk unit tests; highlight smoke tests are medium-risk due fixture selection.

## 8. Execution Plan

- Commands:
  - `cargo test -p highlight-spans`
  - `cargo test -p theme-engine`
  - `cargo test -p render-ansi`
  - `cargo test`
- Expected pass criteria:
  - All existing and newly added scenario tests pass.
  - No regression in current tests.
- Failure triage rules:
  - First failing scenario ID blocks sign-off for corresponding area.
  - P0 failures must be fixed before release.

## 9. Exit Criteria

- All `P0` scenarios are covered by passing tests.
- At least 75% of `P1` scenarios are covered, or deferred with explicit release rationale.
- No unresolved failures in highlight/theme/render critical path tests.

## 10. Deferred Scenarios

- None currently deferred in planning phase.
- Trigger for deferral acceptance: scenario requires architectural change or unstable external dependency behavior.

## Assumptions

- Test fixtures can be authored without introducing external files.
- Existing tests remain deterministic across local and CI environments.

## Open Questions

- Should alias behavior for built-in theme names be treated as stable API contract or convenience behavior?
- Is lossy UTF-8 conversion in renderer an intentional compatibility requirement?

## Evidence

- `crates/highlight-spans/src/lib.rs:63`
- `crates/highlight-spans/src/lib.rs:112`
- `crates/highlight-spans/src/lib.rs:142`
- `crates/highlight-spans/src/lib.rs:161`
- `crates/theme-engine/src/lib.rs:117`
- `crates/theme-engine/src/lib.rs:125`
- `crates/theme-engine/src/lib.rs:131`
- `crates/theme-engine/src/lib.rs:148`
- `crates/theme-engine/src/lib.rs:292`
- `crates/render-ansi/src/lib.rs:53`
- `crates/render-ansi/src/lib.rs:77`
- `crates/render-ansi/src/lib.rs:217`
- `crates/render-ansi/src/lib.rs:227`
- `crates/render-ansi/src/lib.rs:264`
- `crates/render-ansi/src/lib.rs:282`
