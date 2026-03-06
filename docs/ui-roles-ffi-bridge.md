# UI Roles, C FFI, and Auto Bridge Mode (Concise)

## UI Roles

UI roles are theme keys for non-syntax surfaces (status bars, tabs, selection, terminal defaults), separate from capture styles like `@keyword`.

Supported roles:

- `default_fg`
- `default_bg`
- `statusline`
- `statusline_inactive`
- `tab_active`
- `tab_inactive`
- `selection`
- `cursorline`

Resolution behavior:

- `Theme::resolve_ui("...")` checks the dedicated `ui` map first.
- Then it falls back to legacy keys in `styles` for compatibility.
- `default_terminal_colors()` resolves `default_fg/default_bg`, then falls back to `styles.normal`.

Practical use:

- Use UI roles to theme host UI parts (status bar, tab strip, selection highlight).
- Use `default_terminal_colors()` with OSC 10/11 if you want to set terminal session defaults.

## C / FFI Surface

`theme-engine-ffi` exposes theme resolution to C hosts.

Core C API functions:

- `theme_engine_theme_load_builtin`
- `theme_engine_theme_load_json`
- `theme_engine_theme_resolve_capture`
- `theme_engine_theme_resolve_ui`
- `theme_engine_theme_default_terminal_colors`
- `theme_engine_theme_free`

Returned style payload (`ThemeEngineStyle`):

- `has_fg`, `fg`
- `has_bg`, `bg`
- `bold`, `italic`, `underline`

Typical C-host flow:

1. Load theme once (`load_builtin` or `load_json`).
2. Resolve syntax captures for token painting (`resolve_capture`).
3. Resolve UI roles for chrome (`resolve_ui`).
4. Read terminal defaults for OSC (`default_terminal_colors`).
5. Free handle at shutdown (`theme_free`).

## `vt_patch_bridge` Automatic Mode

`vt_patch_bridge` now chooses renderer mode automatically from CLI args/input shape.

Mode selection:

- With `--origin-row`: uses `IncrementalRenderer` (multiline viewport diff mode).
- Without `--origin-row` and single-line snapshots: uses `StreamLineRenderer`.
- Without `--origin-row` and multiline snapshots: uses full-rerender fallback.

Full-rerender fallback behavior:

- Emits relative clear/reposition control sequences.
- Repaints full highlighted output.
- Uses logical `\n` line count (from `--prev` when provided) for clearing.
- Background output toggle: use `set_preserve_terminal_background(true|false)` in API code, or `--terminal-bg` (preserve terminal background) / `--theme-bg` (emit themed background) in bridge CLI.

Quick examples:

```bash
# Incremental viewport mode
cargo run -p render-ansi --example vt_patch_bridge -- \
  /tmp/new.sql tokyonight-dark sql \
  --prev /tmp/old.sql \
  --origin-row 4 --origin-col 7 --width 120 --height 40

# Auto stream-line mode (single line)
cargo run -p render-ansi --example vt_patch_bridge -- \
  new.mac tokyonight-dark objectscript --prev old.mac

# Auto full-rerender fallback (multiline, no origin)
cargo run -p render-ansi --example vt_patch_bridge -- \
  /tmp/new.sql tokyonight-dark sql --prev /tmp/old.sql
```
