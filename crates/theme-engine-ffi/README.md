# theme-engine-ffi

`theme-engine-ffi` exposes `theme-engine` to C hosts.

It supports:

- loading built-in themes (`theme_engine_theme_load_builtin`)
- loading JSON themes (`theme_engine_theme_load_json`)
- resolving syntax capture styles (`theme_engine_theme_resolve_capture`)
- resolving UI role styles (`theme_engine_theme_resolve_ui`) such as `statusline`, `tab_active`, `selection`
- reading default terminal fg/bg (`theme_engine_theme_default_terminal_colors`) for terminal default-color integration (OSC 10/11)

See [`include/theme_engine_ffi.h`](include/theme_engine_ffi.h) for the C ABI.
