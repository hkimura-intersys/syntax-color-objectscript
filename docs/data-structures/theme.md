# Data Structure: Theme

## Overview

- `Theme` stores capture-name to `Style` mappings and resolves style lookups with normalization and fallback (`crates/theme-engine/src/lib.rs:82`, `crates/theme-engine/src/lib.rs:117`).
- Primary consumers are renderer adapters and host code selecting built-in or custom theme documents (`crates/render-ansi/src/lib.rs:35`, `crates/theme-engine/src/lib.rs:160`).

## Scope

- In scope: `theme-engine` type definitions and lookup behavior (`crates/theme-engine/src/lib.rs:67`, `crates/theme-engine/src/lib.rs:82`).
- Out of scope: parser capture generation and terminal escape emission (`crates/highlight-spans/src/lib.rs:63`, `crates/render-ansi/src/lib.rs:163`).

## Canonical Definition

- Canonical definition: `Theme` wraps `styles: BTreeMap<String, Style>` (`crates/theme-engine/src/lib.rs:82`, `crates/theme-engine/src/lib.rs:83`).
- Supporting definitions: `Style` (`fg`, `bg`, `bold`, `italic`, `underline`) and `Rgb` (`r`, `g`, `b`) (`crates/theme-engine/src/lib.rs:53`, `crates/theme-engine/src/lib.rs:67`).

## Fields and Types

- `Theme.styles`: `BTreeMap<String, Style>` (`crates/theme-engine/src/lib.rs:83`).
- `Style.fg` and `Style.bg`: optional `Rgb` values (`crates/theme-engine/src/lib.rs:70`, `crates/theme-engine/src/lib.rs:72`).
- `Style.bold`, `Style.italic`, `Style.underline`: boolean flags defaulting to false (`crates/theme-engine/src/lib.rs:73`, `crates/theme-engine/src/lib.rs:74`, `crates/theme-engine/src/lib.rs:76`, `crates/theme-engine/src/lib.rs:78`).
- Capture keys are normalized (trim, optional `@` strip, lowercase) before insert/get/resolve (`crates/theme-engine/src/lib.rs:101`, `crates/theme-engine/src/lib.rs:112`, `crates/theme-engine/src/lib.rs:193`).

## Invariants

- Keys inside the map are normalized capture names (`crates/theme-engine/src/lib.rs:103`, `crates/theme-engine/src/lib.rs:193`).
- `resolve` falls back by trimming dotted suffixes, then to `normal` (`crates/theme-engine/src/lib.rs:125`, `crates/theme-engine/src/lib.rs:131`).
- Built-in theme names are constrained to known constants/aliases (`crates/theme-engine/src/lib.rs:6`, `crates/theme-engine/src/lib.rs:33`).

## Ownership and Responsibilities

- `theme-engine` owns parsing, normalization, and style-lookup semantics (`crates/theme-engine/src/lib.rs:134`, `crates/theme-engine/src/lib.rs:139`, `crates/theme-engine/src/lib.rs:117`).
- Renderers consume `Theme` and apply resulting styles to text segments (`crates/render-ansi/src/lib.rs:35`, `crates/render-ansi/src/lib.rs:47`).

## Lifecycle

- Creation path: `Theme::new`, `from_styles`, `from_json_str`, `from_toml_str`, or built-in loaders (`crates/theme-engine/src/lib.rs:88`, `crates/theme-engine/src/lib.rs:93`, `crates/theme-engine/src/lib.rs:134`, `crates/theme-engine/src/lib.rs:144`).
- Update path: `insert` normalizes key and replaces existing style when key already exists (`crates/theme-engine/src/lib.rs:101`, `crates/theme-engine/src/lib.rs:103`).
- Deletion/retention path: no explicit delete API; callers can replace the whole `Theme` object (`crates/theme-engine/src/lib.rs:82`).

## Update and Maintenance

- Primary updates happen by editing built-in JSON files or custom input docs parsed at runtime (`crates/theme-engine/src/lib.rs:45`, `crates/theme-engine/src/lib.rs:134`).
- Background jobs or batch processes: Not applicable (the module exposes synchronous in-process constructors/loaders only: `crates/theme-engine/src/lib.rs:134`, `crates/theme-engine/src/lib.rs:160`).
- Migration strategy: preserve normalized key contract and `normal` fallback for backward-compatible theme behavior (`crates/theme-engine/src/lib.rs:131`, `crates/theme-engine/src/lib.rs:193`).

## Storage and Access

- Stored in-memory in a `BTreeMap`, giving deterministic key ordering for serialized/debug access (`crates/theme-engine/src/lib.rs:1`, `crates/theme-engine/src/lib.rs:83`).
- Access paths: `get_exact` for direct lookup; `resolve` for hierarchical fallback (`crates/theme-engine/src/lib.rs:112`, `crates/theme-engine/src/lib.rs:117`).

## APIs and Interfaces

- Public APIs: `insert`, `styles`, `get_exact`, `resolve`, and constructors (`crates/theme-engine/src/lib.rs:101`, `crates/theme-engine/src/lib.rs:107`, `crates/theme-engine/src/lib.rs:117`).
- Built-in interfaces: `available_themes`, `load_theme`, and enum-based `from_builtin` (`crates/theme-engine/src/lib.rs:155`, `crates/theme-engine/src/lib.rs:160`, `crates/theme-engine/src/lib.rs:144`).

## Usage Examples

- Built-in theme loading: `load_theme("tokyo-night")` alias support is tested (`crates/theme-engine/src/lib.rs:160`, `crates/theme-engine/src/lib.rs:298`).
- Fallback behavior: resolving `@comment.documentation` to `comment`, then to `normal` is tested (`crates/theme-engine/src/lib.rs:213`, `crates/theme-engine/src/lib.rs:235`).

## Pitfalls and Edge Cases

- Unknown built-in names return `ThemeError::UnknownBuiltinTheme` (`crates/theme-engine/src/lib.rs:150`, `crates/theme-engine/src/lib.rs:173`).
- Missing `normal` style means unresolved unknown captures may return `None` (`crates/theme-engine/src/lib.rs:131`).
- Input format mismatch surfaces serde parse errors for JSON/TOML loaders (`crates/theme-engine/src/lib.rs:167`, `crates/theme-engine/src/lib.rs:169`).

## Observability

Not applicable.

No built-in logging, metrics, or tracing for theme resolution paths (`crates/theme-engine/src/lib.rs:117`).

## Security and Privacy

- Theme data contains color and style metadata only (`crates/theme-engine/src/lib.rs:67`).
- Input parsing uses serde; malformed theme text is rejected with typed parse errors (`crates/theme-engine/src/lib.rs:167`, `crates/theme-engine/src/lib.rs:169`).

## Assumptions

- Consumers include a `normal` style for predictable fallback behavior.
- Runtime callers treat `Theme` as immutable after setup, even though insertion is supported.
