#ifndef THEME_ENGINE_FFI_H
#define THEME_ENGINE_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

enum {
  SYNTAX_COLOR_FFI_OK = 0,
  SYNTAX_COLOR_FFI_ERR_NULL = 1,
  SYNTAX_COLOR_FFI_ERR_UTF8 = 2,
  SYNTAX_COLOR_FFI_ERR_THEME = 3,
  SYNTAX_COLOR_FFI_ERR_NOT_FOUND = 4,
  SYNTAX_COLOR_FFI_ERR_HIGHLIGHT = 5,
  SYNTAX_COLOR_FFI_ERR_RENDER = 6,
  SYNTAX_COLOR_FFI_ERR_INVALID_ARGUMENT = 7
};

enum {
  THEME_ENGINE_FFI_OK = SYNTAX_COLOR_FFI_OK,
  THEME_ENGINE_FFI_ERR_NULL = SYNTAX_COLOR_FFI_ERR_NULL,
  THEME_ENGINE_FFI_ERR_UTF8 = SYNTAX_COLOR_FFI_ERR_UTF8,
  THEME_ENGINE_FFI_ERR_THEME = SYNTAX_COLOR_FFI_ERR_THEME,
  THEME_ENGINE_FFI_ERR_NOT_FOUND = SYNTAX_COLOR_FFI_ERR_NOT_FOUND
};

typedef enum SyntaxColorGrammar {
  SYNTAX_COLOR_GRAMMAR_OBJECTSCRIPT = 0,
  SYNTAX_COLOR_GRAMMAR_OBJECTSCRIPT_ROUTINE = 1,
  SYNTAX_COLOR_GRAMMAR_SQL = 2,
  SYNTAX_COLOR_GRAMMAR_PYTHON = 3,
  SYNTAX_COLOR_GRAMMAR_MARKDOWN = 4,
  SYNTAX_COLOR_GRAMMAR_MDX = 5,
  SYNTAX_COLOR_GRAMMAR_XML = 6,
  SYNTAX_COLOR_GRAMMAR_JSON = 7,
  SYNTAX_COLOR_GRAMMAR_YAML = 8
} SyntaxColorGrammar;

typedef enum SyntaxColorColorMode {
  SYNTAX_COLOR_COLOR_MODE_TRUECOLOR = 0,
  SYNTAX_COLOR_COLOR_MODE_ANSI256 = 1,
  SYNTAX_COLOR_COLOR_MODE_ANSI16 = 2
} SyntaxColorColorMode;

typedef struct ThemeEngineTheme ThemeEngineTheme;
typedef struct SyntaxColorHighlighter SyntaxColorHighlighter;
typedef struct SyntaxColorIncrementalRenderer SyntaxColorIncrementalRenderer;
typedef struct SyntaxColorStreamLineRenderer SyntaxColorStreamLineRenderer;

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

typedef struct SyntaxColorString {
  uint8_t *data;
  size_t len;
} SyntaxColorString;

typedef struct SyntaxColorStringArray {
  SyntaxColorString *items;
  size_t count;
} SyntaxColorStringArray;

typedef struct SyntaxColorAttr {
  size_t id;
  SyntaxColorString capture_name;
} SyntaxColorAttr;

typedef struct SyntaxColorSpan {
  size_t attr_id;
  size_t start_byte;
  size_t end_byte;
} SyntaxColorSpan;

typedef struct SyntaxColorHighlightResult {
  SyntaxColorAttr *attrs;
  size_t attr_count;
  SyntaxColorSpan *spans;
  size_t span_count;
} SyntaxColorHighlightResult;

typedef struct SyntaxColorStyledSpan {
  size_t start_byte;
  size_t end_byte;
  uint8_t has_style;
  ThemeEngineStyle style;
} SyntaxColorStyledSpan;

typedef struct SyntaxColorStyledSpanBuffer {
  SyntaxColorStyledSpan *spans;
  size_t count;
} SyntaxColorStyledSpanBuffer;

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

void syntax_color_string_free(SyntaxColorString *value);
void syntax_color_string_array_free(SyntaxColorStringArray *values);
void syntax_color_highlight_result_free(SyntaxColorHighlightResult *result);
void syntax_color_styled_spans_free(SyntaxColorStyledSpanBuffer *spans);

int32_t syntax_color_highlighter_new(SyntaxColorHighlighter **out_highlighter);
void syntax_color_highlighter_free(SyntaxColorHighlighter *highlighter);

int32_t syntax_color_highlighter_highlight(
    SyntaxColorHighlighter *highlighter,
    const uint8_t *source,
    size_t source_len,
    int32_t grammar,
    SyntaxColorHighlightResult *out_result);

int32_t syntax_color_resolve_styled_spans(
    const ThemeEngineTheme *theme,
    size_t source_len,
    const SyntaxColorHighlightResult *highlight,
    uint8_t fill_uncovered_with_normal,
    SyntaxColorStyledSpanBuffer *out_spans);

int32_t syntax_color_render_ansi(
    const uint8_t *source,
    size_t source_len,
    const SyntaxColorStyledSpan *spans,
    size_t span_count,
    int32_t color_mode,
    uint8_t preserve_terminal_background,
    SyntaxColorString *out_ansi);

int32_t syntax_color_render_ansi_lines(
    const uint8_t *source,
    size_t source_len,
    const SyntaxColorStyledSpan *spans,
    size_t span_count,
    int32_t color_mode,
    uint8_t preserve_terminal_background,
    SyntaxColorStringArray *out_lines);

int32_t syntax_color_highlight_to_ansi(
    SyntaxColorHighlighter *highlighter,
    const ThemeEngineTheme *theme,
    const uint8_t *source,
    size_t source_len,
    int32_t grammar,
    int32_t color_mode,
    uint8_t preserve_terminal_background,
    SyntaxColorString *out_ansi);

int32_t syntax_color_highlight_to_ansi_lines(
    SyntaxColorHighlighter *highlighter,
    const ThemeEngineTheme *theme,
    const uint8_t *source,
    size_t source_len,
    int32_t grammar,
    int32_t color_mode,
    uint8_t preserve_terminal_background,
    SyntaxColorStringArray *out_lines);

int32_t syntax_color_osc_set_default_colors(
    const ThemeEngineTheme *theme,
    SyntaxColorString *out_osc);

int32_t syntax_color_osc_reset_default_foreground(SyntaxColorString *out_osc);
int32_t syntax_color_osc_reset_default_background(SyntaxColorString *out_osc);
int32_t syntax_color_osc_reset_default_colors(SyntaxColorString *out_osc);

int32_t syntax_color_incremental_renderer_new(
    size_t width,
    size_t height,
    SyntaxColorIncrementalRenderer **out_renderer);

void syntax_color_incremental_renderer_free(SyntaxColorIncrementalRenderer *renderer);

int32_t syntax_color_incremental_renderer_resize(
    SyntaxColorIncrementalRenderer *renderer,
    size_t width,
    size_t height);

int32_t syntax_color_incremental_renderer_clear_state(
    SyntaxColorIncrementalRenderer *renderer);

int32_t syntax_color_incremental_renderer_set_origin(
    SyntaxColorIncrementalRenderer *renderer,
    size_t row,
    size_t col);

int32_t syntax_color_incremental_renderer_set_color_mode(
    SyntaxColorIncrementalRenderer *renderer,
    int32_t color_mode);

int32_t syntax_color_incremental_renderer_set_preserve_terminal_background(
    SyntaxColorIncrementalRenderer *renderer,
    uint8_t preserve_terminal_background);

int32_t syntax_color_incremental_renderer_render_patch(
    SyntaxColorIncrementalRenderer *renderer,
    const uint8_t *source,
    size_t source_len,
    const SyntaxColorStyledSpan *spans,
    size_t span_count,
    SyntaxColorString *out_patch);

int32_t syntax_color_incremental_renderer_highlight_to_patch(
    SyntaxColorIncrementalRenderer *renderer,
    SyntaxColorHighlighter *highlighter,
    const ThemeEngineTheme *theme,
    const uint8_t *source,
    size_t source_len,
    int32_t grammar,
    SyntaxColorString *out_patch);

int32_t syntax_color_stream_line_renderer_new(
    SyntaxColorStreamLineRenderer **out_renderer);

void syntax_color_stream_line_renderer_free(SyntaxColorStreamLineRenderer *renderer);

int32_t syntax_color_stream_line_renderer_clear_state(
    SyntaxColorStreamLineRenderer *renderer);

int32_t syntax_color_stream_line_renderer_set_color_mode(
    SyntaxColorStreamLineRenderer *renderer,
    int32_t color_mode);

int32_t syntax_color_stream_line_renderer_set_preserve_terminal_background(
    SyntaxColorStreamLineRenderer *renderer,
    uint8_t preserve_terminal_background);

int32_t syntax_color_stream_line_renderer_render_line_patch(
    SyntaxColorStreamLineRenderer *renderer,
    const uint8_t *source,
    size_t source_len,
    const SyntaxColorStyledSpan *spans,
    size_t span_count,
    SyntaxColorString *out_patch);

int32_t syntax_color_stream_line_renderer_highlight_line_to_patch(
    SyntaxColorStreamLineRenderer *renderer,
    SyntaxColorHighlighter *highlighter,
    const ThemeEngineTheme *theme,
    const uint8_t *source,
    size_t source_len,
    int32_t grammar,
    SyntaxColorString *out_patch);

#ifdef __cplusplus
}
#endif

#endif
