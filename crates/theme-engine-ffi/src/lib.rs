use std::ffi::{c_char, CStr};

use theme_engine::{load_theme, Rgb, Style, Theme};

/// Operation succeeded.
pub const THEME_ENGINE_FFI_OK: i32 = 0;
/// A required pointer argument was null.
pub const THEME_ENGINE_FFI_ERR_NULL: i32 = 1;
/// A C string argument was not valid UTF-8.
pub const THEME_ENGINE_FFI_ERR_UTF8: i32 = 2;
/// Theme loading or parsing failed.
pub const THEME_ENGINE_FFI_ERR_THEME: i32 = 3;
/// Requested style was not found.
pub const THEME_ENGINE_FFI_ERR_NOT_FOUND: i32 = 4;

/// Opaque C handle for a loaded theme.
pub struct ThemeEngineTheme {
    theme: Theme,
}

/// C ABI RGB triplet.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct ThemeEngineRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// C ABI style payload.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct ThemeEngineStyle {
    pub has_fg: u8,
    pub fg: ThemeEngineRgb,
    pub has_bg: u8,
    pub bg: ThemeEngineRgb,
    pub bold: u8,
    pub italic: u8,
    pub underline: u8,
}

fn rgb_to_ffi(rgb: Rgb) -> ThemeEngineRgb {
    ThemeEngineRgb {
        r: rgb.r,
        g: rgb.g,
        b: rgb.b,
    }
}

fn style_to_ffi(style: Style) -> ThemeEngineStyle {
    let (has_fg, fg) = if let Some(color) = style.fg {
        (1, rgb_to_ffi(color))
    } else {
        (0, ThemeEngineRgb::default())
    };
    let (has_bg, bg) = if let Some(color) = style.bg {
        (1, rgb_to_ffi(color))
    } else {
        (0, ThemeEngineRgb::default())
    };

    ThemeEngineStyle {
        has_fg,
        fg,
        has_bg,
        bg,
        bold: u8::from(style.bold),
        italic: u8::from(style.italic),
        underline: u8::from(style.underline),
    }
}

unsafe fn parse_cstr<'a>(value: *const c_char) -> Result<&'a str, i32> {
    if value.is_null() {
        return Err(THEME_ENGINE_FFI_ERR_NULL);
    }
    // SAFETY: validated non-null above; caller promises valid C string lifetime.
    let cstr = unsafe { CStr::from_ptr(value) };
    cstr.to_str().map_err(|_| THEME_ENGINE_FFI_ERR_UTF8)
}

/// Frees a theme handle previously returned by this library.
#[no_mangle]
pub unsafe extern "C" fn theme_engine_theme_free(theme: *mut ThemeEngineTheme) {
    if theme.is_null() {
        return;
    }
    // SAFETY: pointer originated from Box::into_raw in this library.
    let _ = unsafe { Box::from_raw(theme) };
}

/// Loads a built-in theme by name (for example `"tokyonight-dark"`).
///
/// Returns status code and writes a new handle to `out_theme` on success.
#[no_mangle]
pub unsafe extern "C" fn theme_engine_theme_load_builtin(
    name: *const c_char,
    out_theme: *mut *mut ThemeEngineTheme,
) -> i32 {
    if out_theme.is_null() {
        return THEME_ENGINE_FFI_ERR_NULL;
    }
    // SAFETY: parse_cstr validates null and UTF-8.
    let name = match unsafe { parse_cstr(name) } {
        Ok(name) => name,
        Err(code) => return code,
    };

    let theme = match load_theme(name) {
        Ok(theme) => theme,
        Err(_) => return THEME_ENGINE_FFI_ERR_THEME,
    };

    let boxed = Box::new(ThemeEngineTheme { theme });
    // SAFETY: out_theme non-null checked above.
    unsafe { *out_theme = Box::into_raw(boxed) };
    THEME_ENGINE_FFI_OK
}

/// Loads a theme from a JSON string and returns a new handle.
#[no_mangle]
pub unsafe extern "C" fn theme_engine_theme_load_json(
    json: *const c_char,
    out_theme: *mut *mut ThemeEngineTheme,
) -> i32 {
    if out_theme.is_null() {
        return THEME_ENGINE_FFI_ERR_NULL;
    }
    // SAFETY: parse_cstr validates null and UTF-8.
    let json = match unsafe { parse_cstr(json) } {
        Ok(json) => json,
        Err(code) => return code,
    };

    let theme = match Theme::from_json_str(json) {
        Ok(theme) => theme,
        Err(_) => return THEME_ENGINE_FFI_ERR_THEME,
    };

    let boxed = Box::new(ThemeEngineTheme { theme });
    // SAFETY: out_theme non-null checked above.
    unsafe { *out_theme = Box::into_raw(boxed) };
    THEME_ENGINE_FFI_OK
}

/// Resolves a syntax capture style (for example `"@keyword"`).
#[no_mangle]
pub unsafe extern "C" fn theme_engine_theme_resolve_capture(
    theme: *const ThemeEngineTheme,
    capture_name: *const c_char,
    out_style: *mut ThemeEngineStyle,
) -> i32 {
    if theme.is_null() || out_style.is_null() {
        return THEME_ENGINE_FFI_ERR_NULL;
    }
    // SAFETY: parse_cstr validates null and UTF-8.
    let capture_name = match unsafe { parse_cstr(capture_name) } {
        Ok(name) => name,
        Err(code) => return code,
    };
    // SAFETY: theme pointer checked non-null above.
    let theme = unsafe { &*theme };

    let Some(style) = theme.theme.resolve(capture_name).copied() else {
        return THEME_ENGINE_FFI_ERR_NOT_FOUND;
    };
    // SAFETY: out_style checked non-null above.
    unsafe { *out_style = style_to_ffi(style) };
    THEME_ENGINE_FFI_OK
}

/// Resolves a UI role style (for example `"statusline"` or `"tab_active"`).
#[no_mangle]
pub unsafe extern "C" fn theme_engine_theme_resolve_ui(
    theme: *const ThemeEngineTheme,
    role_name: *const c_char,
    out_style: *mut ThemeEngineStyle,
) -> i32 {
    if theme.is_null() || out_style.is_null() {
        return THEME_ENGINE_FFI_ERR_NULL;
    }
    // SAFETY: parse_cstr validates null and UTF-8.
    let role_name = match unsafe { parse_cstr(role_name) } {
        Ok(name) => name,
        Err(code) => return code,
    };
    // SAFETY: theme pointer checked non-null above.
    let theme = unsafe { &*theme };

    let Some(style) = theme.theme.resolve_ui(role_name) else {
        return THEME_ENGINE_FFI_ERR_NOT_FOUND;
    };
    // SAFETY: out_style checked non-null above.
    unsafe { *out_style = style_to_ffi(style) };
    THEME_ENGINE_FFI_OK
}

/// Returns default terminal foreground/background colors from the theme.
///
/// Writes `0/1` presence flags to `out_has_fg` and `out_has_bg`.
/// If a flag is `1`, the corresponding `out_fg`/`out_bg` value is written.
#[no_mangle]
pub unsafe extern "C" fn theme_engine_theme_default_terminal_colors(
    theme: *const ThemeEngineTheme,
    out_has_fg: *mut u8,
    out_fg: *mut ThemeEngineRgb,
    out_has_bg: *mut u8,
    out_bg: *mut ThemeEngineRgb,
) -> i32 {
    if theme.is_null() || out_has_fg.is_null() || out_has_bg.is_null() {
        return THEME_ENGINE_FFI_ERR_NULL;
    }
    // SAFETY: theme pointer checked non-null above.
    let theme = unsafe { &*theme };
    let (fg, bg) = theme.theme.default_terminal_colors();

    // SAFETY: out_has_* pointers checked non-null above.
    unsafe {
        *out_has_fg = u8::from(fg.is_some());
        *out_has_bg = u8::from(bg.is_some());
    }

    if let Some(color) = fg {
        if out_fg.is_null() {
            return THEME_ENGINE_FFI_ERR_NULL;
        }
        // SAFETY: validated non-null when fg exists.
        unsafe { *out_fg = rgb_to_ffi(color) };
    }
    if let Some(color) = bg {
        if out_bg.is_null() {
            return THEME_ENGINE_FFI_ERR_NULL;
        }
        // SAFETY: validated non-null when bg exists.
        unsafe { *out_bg = rgb_to_ffi(color) };
    }

    THEME_ENGINE_FFI_OK
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::ptr;

    use super::{
        theme_engine_theme_default_terminal_colors, theme_engine_theme_free,
        theme_engine_theme_load_builtin, theme_engine_theme_resolve_ui, ThemeEngineRgb,
        ThemeEngineStyle, ThemeEngineTheme, THEME_ENGINE_FFI_OK,
    };

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

        // SAFETY: handle was allocated by load function above.
        unsafe { theme_engine_theme_free(handle) };
    }
}
