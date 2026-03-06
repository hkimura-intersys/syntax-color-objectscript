# Data Structure: Theme

## Overview

- `Theme` stores both syntax capture styles and UI-role styles, with normalized lookup and fallback behavior (`crates/theme-engine/src/lib.rs:145`, `crates/theme-engine/src/lib.rs:246`).
- Primary consumers are ANSI render adapters and host-side paint engines (`crates/render-ansi/src/lib.rs:137`, `crates/render-ansi/src/lib.rs:652`).

## Scope

- In scope: `theme-engine` structures and lookup APIs (`crates/theme-engine/src/lib.rs:84`, `crates/theme-engine/src/lib.rs:145`).
- Out of scope: syntax capture generation and renderer-specific escape emission (`crates/highlight-spans/src/lib.rs:118`, `crates/render-ansi/src/lib.rs:652`).

## Canonical Definition

- Canonical definition: `Theme` wraps:
  - `styles: BTreeMap<String, Style>` for syntax captures
  - `ui: BTreeMap<String, Style>` for UI roles
  (`crates/theme-engine/src/lib.rs:146`, `crates/theme-engine/src/lib.rs:148`).
- Supporting definitions:
  - `Style` (`fg`, `bg`, `bold`, `italic`, `underline`) (`crates/theme-engine/src/lib.rs:84`)
  - `Rgb` (`r`, `g`, `b`) (`crates/theme-engine/src/lib.rs:69`)
  - `UiRole` (`default_fg`, `default_bg`, `statusline`, `tab_*`, etc.) (`crates/theme-engine/src/lib.rs:98`)

## Fields and Types

- `Theme.styles`: normalized syntax-style map (`crates/theme-engine/src/lib.rs:147`, `crates/theme-engine/src/lib.rs:188`).
- `Theme.ui`: normalized UI-role style map (`crates/theme-engine/src/lib.rs:148`, `crates/theme-engine/src/lib.rs:203`).
- `Style.fg` and `Style.bg`: optional `Rgb` colors (`crates/theme-engine/src/lib.rs:87`, `crates/theme-engine/src/lib.rs:89`).
- `Style.bold`, `Style.italic`, `Style.underline`: boolean flags (`crates/theme-engine/src/lib.rs:91`, `crates/theme-engine/src/lib.rs:95`).
- Keys are normalized (trim, optional `@` strip, lowercase) before insert/get/resolve (`crates/theme-engine/src/lib.rs:181`, `crates/theme-engine/src/lib.rs:196`, `crates/theme-engine/src/lib.rs:438`).

## Invariants

- Keys stored in both maps are normalized names (`crates/theme-engine/src/lib.rs:183`, `crates/theme-engine/src/lib.rs:198`).
- `resolve` fallback order is dotted parent trimming, then `normal` (`crates/theme-engine/src/lib.rs:224`, `crates/theme-engine/src/lib.rs:238`).
- `resolve_ui` checks dedicated `ui`, then compatibility fallbacks in `styles`, then typed role fallback (`crates/theme-engine/src/lib.rs:246`, `crates/theme-engine/src/lib.rs:264`).
- Default terminal fg/bg comes from UI roles (`default_fg`, `default_bg`) with fallback to `styles.normal` (`crates/theme-engine/src/lib.rs:326`).

## Ownership and Responsibilities

- `theme-engine` owns parse/normalize/lookup semantics for both syntax and UI roles (`crates/theme-engine/src/lib.rs:345`, `crates/theme-engine/src/lib.rs:246`).
- Renderers and host bridges consume resolved styles; they do not own theme fallback logic (`crates/render-ansi/src/lib.rs:99`, `crates/theme-engine-ffi/src/lib.rs:191`).

## Lifecycle

- Creation path: `Theme::new`, `from_styles`, `from_parts`, `from_json_str`, `from_toml_str`, and built-in loaders (`crates/theme-engine/src/lib.rs:154`, `crates/theme-engine/src/lib.rs:160`, `crates/theme-engine/src/lib.rs:166`, `crates/theme-engine/src/lib.rs:345`, `crates/theme-engine/src/lib.rs:365`).
- Update path: `insert` and `insert_ui` normalize and replace existing entries (`crates/theme-engine/src/lib.rs:181`, `crates/theme-engine/src/lib.rs:196`).
- Deletion path: no explicit delete API; replace the `Theme` value when needed (`crates/theme-engine/src/lib.rs:145`).

## Update and Maintenance

- Built-in theme updates happen by editing JSON assets under `crates/theme-engine/themes` and reloading (`crates/theme-engine/src/lib.rs:57`).
- Runtime theme docs can use wrapped schema (`styles` + optional `ui`) or flat legacy styles (`crates/theme-engine/src/lib.rs:408`, `crates/theme-engine/src/lib.rs:419`).
- Migration strategy keeps legacy compatibility: UI role resolution falls back to legacy keys in `styles` (`crates/theme-engine/src/lib.rs:243`, `crates/theme-engine/src/lib.rs:286`).

## Storage and Access

- Stored in-memory as two `BTreeMap`s for deterministic key order (`crates/theme-engine/src/lib.rs:1`, `crates/theme-engine/src/lib.rs:146`).
- Access paths:
  - Syntax: `get_exact`, `resolve` (`crates/theme-engine/src/lib.rs:209`, `crates/theme-engine/src/lib.rs:224`)
  - UI: `get_ui_exact`, `resolve_ui`, `resolve_ui_role` (`crates/theme-engine/src/lib.rs:215`, `crates/theme-engine/src/lib.rs:246`, `crates/theme-engine/src/lib.rs:264`)
  - Terminal defaults: `default_terminal_colors` (`crates/theme-engine/src/lib.rs:326`)

## APIs and Interfaces

- Public APIs include constructors, insert/get/resolve methods, and default terminal-color helpers (`crates/theme-engine/src/lib.rs:151`, `crates/theme-engine/src/lib.rs:326`).
- Built-in interfaces: `available_themes`, `load_theme`, and enum-based loading (`crates/theme-engine/src/lib.rs:381`, `crates/theme-engine/src/lib.rs:392`).
- FFI surface exposes capture lookup, UI-role lookup, and default terminal fg/bg (`crates/theme-engine-ffi/src/lib.rs:149`, `crates/theme-engine-ffi/src/lib.rs:175`, `crates/theme-engine-ffi/src/lib.rs:204`).

## Usage Examples

- Syntax lookup fallback (`comment.documentation -> comment -> normal`) is covered by tests (`crates/theme-engine/src/lib.rs:460`).
- UI-role fallback (`tab_active`, `tab_inactive`, `statusline`) is covered by tests (`crates/theme-engine/src/lib.rs:632`).
- Wrapped theme docs with explicit `ui` are covered by tests (`crates/theme-engine/src/lib.rs:588`).

## Pitfalls and Edge Cases

- Unknown built-in names return `ThemeError::UnknownBuiltinTheme` (`crates/theme-engine/src/lib.rs:405`).
- Missing `normal` can cause unresolved syntax captures and weaker default terminal-color fallback (`crates/theme-engine/src/lib.rs:238`, `crates/theme-engine/src/lib.rs:330`).
- Unknown UI role names return `None` unless matched by aliases/fallback keys (`crates/theme-engine/src/lib.rs:128`, `crates/theme-engine/src/lib.rs:259`).
- Input format mismatches surface serde parse errors (`crates/theme-engine/src/lib.rs:399`, `crates/theme-engine/src/lib.rs:401`).

## Observability

Not applicable.

No built-in logging/metrics/tracing in theme resolution paths (`crates/theme-engine/src/lib.rs:246`).

## Security and Privacy

- Theme data only contains style metadata (`crates/theme-engine/src/lib.rs:84`).
- Parsing uses serde with typed error returns for malformed input (`crates/theme-engine/src/lib.rs:345`, `crates/theme-engine/src/lib.rs:355`).

## Assumptions

- Consumers usually provide `normal` for predictable syntax and terminal-default fallback.
- UI role entries are optional; missing roles rely on compatibility fallbacks to legacy style keys.
- Hosts treat a loaded `Theme` as immutable configuration during a render session.
