#ifndef THEME_ENGINE_FFI_H
#define THEME_ENGINE_FFI_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

enum {
  THEME_ENGINE_FFI_OK = 0,
  THEME_ENGINE_FFI_ERR_NULL = 1,
  THEME_ENGINE_FFI_ERR_UTF8 = 2,
  THEME_ENGINE_FFI_ERR_THEME = 3,
  THEME_ENGINE_FFI_ERR_NOT_FOUND = 4
};

typedef struct ThemeEngineTheme ThemeEngineTheme;

typedef struct ThemeEngineRgb {
  uint8_t r;
  uint8_t g;
  uint8_t b;
} ThemeEngineRgb;

typedef struct ThemeEngineStyle {
  uint8_t has_fg;
  ThemeEngineRgb fg;
  uint8_t has_bg;
  ThemeEngineRgb bg;
  uint8_t bold;
  uint8_t italic;
  uint8_t underline;
} ThemeEngineStyle;

int32_t theme_engine_theme_load_builtin(const char *name, ThemeEngineTheme **out_theme);
int32_t theme_engine_theme_load_json(const char *json, ThemeEngineTheme **out_theme);
void theme_engine_theme_free(ThemeEngineTheme *theme);

int32_t theme_engine_theme_resolve_capture(
    const ThemeEngineTheme *theme,
    const char *capture_name,
    ThemeEngineStyle *out_style);

int32_t theme_engine_theme_resolve_ui(
    const ThemeEngineTheme *theme,
    const char *role_name,
    ThemeEngineStyle *out_style);

int32_t theme_engine_theme_default_terminal_colors(
    const ThemeEngineTheme *theme,
    uint8_t *out_has_fg,
    ThemeEngineRgb *out_fg,
    uint8_t *out_has_bg,
    ThemeEngineRgb *out_bg);

#ifdef __cplusplus
}
#endif

#endif
