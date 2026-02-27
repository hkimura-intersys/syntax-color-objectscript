# Code Doc Check Report

## Parse Errors

- Mermaid CLI validation script could not complete in this environment within a bounded run window (`__MERMAID_TIMEOUT__`, exit `143` from `validate_mermaid.sh docs`).
- Manual fence check confirms matching open/close Mermaid blocks in `docs/c4.md` and `docs/design-doc.md`.

## Format Issues

- None found.
- `arc42.md` includes required ordered sections, plus `Assumptions`, `Open Questions`, and `Evidence`.
- `c4.md` includes C1, C2, C3, dynamic flow diagrams, and required footer sections.
- `design-doc.md` includes required structure (`Problem`, `Goals`, `Non-Goals`, `Proposal`, `Tradeoffs`) and required additions.

## Missing Citations

- None found in factual statements reviewed.

## Unverifiable Claims

- None found in factual statements reviewed.

## Assumptions

- Assumptions are explicitly listed in each generated core doc.

## Consistency Issues

- None found.
- Component names are consistent across docs: `highlight-spans`, `theme-engine`, `render-ansi`, `HighlightResult`, `Theme`, `StyledSpan`.

## Evidence

- `docs/arc42.md:3`
- `docs/arc42.md:106`
- `docs/c4.md:3`
- `docs/c4.md:119`
- `docs/design-doc.md:3`
- `docs/design-doc.md:73`
- `docs/design-doc.md:83`
