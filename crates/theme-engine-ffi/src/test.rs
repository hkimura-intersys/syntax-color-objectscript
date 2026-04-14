#[cfg(test)]
mod tests {
    use crate::c_structures::*;
    use crate::common::*;
    use std::ffi::CString;
    use std::ptr;

    fn buffer_to_string(buffer: &SyntaxColorString) -> String {
        if buffer.len == 0 {
            return String::new();
        }

        // SAFETY: test buffers come from this crate and are valid for `len` bytes.
        let bytes = unsafe { std::slice::from_raw_parts(buffer.data, buffer.len) };
        String::from_utf8(bytes.to_vec()).expect("buffer should be valid utf-8")
    }

    #[test]
    fn ffi_load_and_resolve_ui_smoke() {
        let name = CString::new("tokyonight-dark").expect("cstring failed");
        let mut handle: *mut ThemeEngineTheme = ptr::null_mut();
        // SAFETY: valid pointers and nul-terminated strings.
        let code = unsafe { theme_engine_theme_load_builtin(name.as_ptr(), &mut handle) };
        assert_eq!(code, THEME_ENGINE_FFI_OK);
        assert!(!handle.is_null());

        let role = CString::new("statusline").expect("cstring failed");
        let mut out_style = ThemeEngineStyle::default();
        // SAFETY: handle and output pointers are valid.
        let code = unsafe { theme_engine_theme_resolve_ui(handle, role.as_ptr(), &mut out_style) };
        assert_eq!(code, THEME_ENGINE_FFI_OK);
        assert_eq!(out_style.has_fg, 1);

        let mut has_fg = 0u8;
        let mut has_bg = 0u8;
        let mut fg = ThemeEngineRgb::default();
        let mut bg = ThemeEngineRgb::default();
        // SAFETY: handle and output pointers are valid.
        let code = unsafe {
            theme_engine_theme_default_terminal_colors(
                handle,
                &mut has_fg,
                &mut fg,
                &mut has_bg,
                &mut bg,
            )
        };
        assert_eq!(code, THEME_ENGINE_FFI_OK);
        assert_eq!(has_fg, 1);
        assert_eq!(has_bg, 1);

        // SAFETY: handle was allocated by the load function above.
        unsafe { theme_engine_theme_free(handle) };
    }

    #[test]
    fn ffi_highlight_returns_attrs_and_spans() {
        let mut highlighter: *mut SyntaxColorHighlighter = ptr::null_mut();
        // SAFETY: output pointer is valid.
        let code = unsafe { syntax_color_highlighter_new(&mut highlighter) };
        assert_eq!(code, SYNTAX_COLOR_FFI_OK);
        assert!(!highlighter.is_null());

        let source = b"SELECT 42";
        let mut result = SyntaxColorHighlightResult::default();
        // SAFETY: handle, input buffer, and output pointer are valid.
        let code = unsafe {
            syntax_color_highlighter_highlight(
                highlighter,
                source.as_ptr(),
                source.len(),
                2,
                &mut result,
            )
        };
        assert_eq!(code, SYNTAX_COLOR_FFI_OK);
        assert!(result.attr_count > 0);
        assert!(result.span_count > 0);

        // SAFETY: result payload was allocated by this crate.
        unsafe { syntax_color_highlight_result_free(&mut result) };
        // SAFETY: handle was allocated by this crate.
        unsafe { syntax_color_highlighter_free(highlighter) };
    }

    #[test]
    fn ffi_highlight_to_ansi_emits_escape_sequences() {
        let name = CString::new("tokyonight-dark").expect("cstring failed");
        let mut theme: *mut ThemeEngineTheme = ptr::null_mut();
        // SAFETY: output pointer and input string are valid.
        let code = unsafe { theme_engine_theme_load_builtin(name.as_ptr(), &mut theme) };
        assert_eq!(code, SYNTAX_COLOR_FFI_OK);

        let source = b"SELECT 42";
        let mut ansi = SyntaxColorString::default();
        // SAFETY: pointers are valid; null highlighter requests a temporary highlighter.
        let code = unsafe {
            syntax_color_highlight_to_ansi(
                ptr::null_mut(),
                theme,
                source.as_ptr(),
                source.len(),
                2,
                0,
                0,
                &mut ansi,
            )
        };
        assert_eq!(code, SYNTAX_COLOR_FFI_OK);

        let rendered = buffer_to_string(&ansi);
        assert!(rendered.contains("\u{1b}["));
        assert!(rendered.contains("SELECT"));

        // SAFETY: buffers and handles were allocated by this crate.
        unsafe { syntax_color_string_free(&mut ansi) };
        unsafe { theme_engine_theme_free(theme) };
    }

    #[test]
    fn ffi_incremental_renderer_tracks_previous_frame() {
        let name = CString::new("tokyonight-dark").expect("cstring failed");
        let mut theme: *mut ThemeEngineTheme = ptr::null_mut();
        // SAFETY: output pointer and input string are valid.
        let code = unsafe { theme_engine_theme_load_builtin(name.as_ptr(), &mut theme) };
        assert_eq!(code, SYNTAX_COLOR_FFI_OK);

        let mut renderer: *mut SyntaxColorIncrementalRenderer = ptr::null_mut();
        // SAFETY: output pointer is valid.
        let code = unsafe { syntax_color_incremental_renderer_new(80, 10, &mut renderer) };
        assert_eq!(code, SYNTAX_COLOR_FFI_OK);

        let source = b"SELECT 42";
        let mut first = SyntaxColorString::default();
        // SAFETY: pointers are valid; null highlighter requests a temporary highlighter.
        let code = unsafe {
            syntax_color_incremental_renderer_highlight_to_patch(
                renderer,
                ptr::null_mut(),
                theme,
                source.as_ptr(),
                source.len(),
                2,
                &mut first,
            )
        };
        assert_eq!(code, SYNTAX_COLOR_FFI_OK);
        assert!(!buffer_to_string(&first).is_empty());

        let mut second = SyntaxColorString::default();
        // SAFETY: same renderer and inputs are still valid.
        let code = unsafe {
            syntax_color_incremental_renderer_highlight_to_patch(
                renderer,
                ptr::null_mut(),
                theme,
                source.as_ptr(),
                source.len(),
                2,
                &mut second,
            )
        };
        assert_eq!(code, SYNTAX_COLOR_FFI_OK);
        assert!(buffer_to_string(&second).is_empty());

        // SAFETY: buffers and handles were allocated by this crate.
        unsafe { syntax_color_string_free(&mut first) };
        unsafe { syntax_color_string_free(&mut second) };
        unsafe { syntax_color_incremental_renderer_free(renderer) };
        unsafe { theme_engine_theme_free(theme) };
    }
}
