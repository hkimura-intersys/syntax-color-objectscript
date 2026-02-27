# Code Health Report

**Audience:** Developers maintaining or extending the codebase.
**Topic:** Evidence-backed critique and optimization recommendations.
**Goal:** Identify non-optimal areas, explain impact, and provide concrete fixes.

## Summary

The workspace is cleanly modular, but two hot-path inefficiencies stand out: repeated full-source allocations in line-oriented APIs and verbose ANSI reset emission for dense highlighting. A smaller maintainability risk is that span ordering/overlap contracts are enforced late (render phase), so invalid upstream spans fail at output time rather than earlier.

## Scope and Inputs

- Scope: Whole repository (workspace crates + tests)
- Repositories or paths reviewed: `Cargo.toml`, `crates/highlight-spans/src/lib.rs`, `crates/theme-engine/src/lib.rs`, `crates/render-ansi/src/lib.rs`
- Architecture docs used (arc42, C4, design doc): `docs/arc42.md`, `docs/c4.md`, `docs/design-doc.md`
- Constraints (runtime, latency, cost, release timeline): Library-first Rust workspace targeting reusable parser/theme/render stages (`Cargo.toml:2`, `Cargo.toml:6`)

## Method

- Evidence-first analysis with file and line references.
- Findings grouped by severity and effort.
- Assumptions explicitly labeled.

## Key Findings

### Finding CH-001: Double Source Materialization in Line APIs

- Evidence: `crates/highlight-spans/src/lib.rs:137`, `crates/highlight-spans/src/lib.rs:142`, `crates/render-ansi/src/lib.rs:153`, `crates/render-ansi/src/lib.rs:154`
- Impact: For large inputs, joining line slices into a temporary full string multiple times increases memory churn and CPU spent on repeated concatenation.
- Suggested change: Introduce a shared helper returning `(source, line_offsets)` once, then reuse for both highlight and render-line phases.
- Effort: M
- Risk: Medium (touches public helper internals and tests)
- Validation: Add benchmark comparing current and optimized `highlight_lines_to_ansi_lines` on large multi-line buffers.

### Finding CH-002: ANSI Reset Emitted Per Styled Segment

- Evidence: `crates/render-ansi/src/lib.rs:168`, `crates/render-ansi/src/lib.rs:171`, `crates/render-ansi/src/lib.rs:175`
- Impact: Dense highlight output may include many short SGR/reset pairs, increasing output size and terminal parsing overhead.
- Suggested change: Add a render mode that coalesces adjacent segments with identical resolved style before emission.
- Effort: M
- Risk: Medium (must preserve exact visual semantics)
- Validation: Snapshot tests for coalesced vs non-coalesced output and output-size comparison.

### Finding CH-003: Span Contract Enforced Late

- Evidence: `crates/render-ansi/src/lib.rs:217`, `crates/render-ansi/src/lib.rs:227`, `crates/render-ansi/src/lib.rs:295`
- Impact: Invalid spans from non-standard producers fail only at render boundaries, making root-cause diagnosis slower.
- Suggested change: Expose and document a reusable span-validation helper for upstream producers, or add optional validation in conversion entrypoints.
- Effort: S
- Risk: Low
- Validation: Add unit tests invoking validation prior to render and asserting identical error variants.

## Outdated Comments

Not applicable.

No stale comments or TODO/FIXME markers were found in reviewed source files.

## Optimization Proposals

- Proposal: Add a compact rendering path that first groups consecutive `StyledSpan` entries by identical `Option<Style>`.
- Evidence: `crates/render-ansi/src/lib.rs:58`, `crates/render-ansi/src/lib.rs:163`, `crates/render-ansi/src/lib.rs:178`
- Expected benefit: Smaller ANSI payloads and reduced terminal control-sequence churn.
- Tradeoffs: Slightly more preprocessing complexity and extra equality checks.
- Validation: Benchmark byte-size and render throughput over representative ObjectScript samples.

## Architectural Drift and Mismatches

Not applicable.

Current implementation aligns with documented split of syntax extraction, theme resolution, and rendering (`docs/arc42.md:42`, `docs/c4.md:35`, `Cargo.toml:2`).

## Prioritized Fix Plan

1. Implement CH-003 span-validation helper and document upstream contract.
2. Refactor line-based APIs to eliminate repeated source joins (CH-001).
3. Add optional style-coalescing ANSI emission mode (CH-002).

## Open Questions

- Question: Should optimization-focused behavior changes remain opt-in to preserve exact output snapshots?
- Why it matters: Existing downstream tests may rely on current escape-sequence granularity.
- Suggested source of truth: Maintainer decision in an ADR-like note under `docs/`.
