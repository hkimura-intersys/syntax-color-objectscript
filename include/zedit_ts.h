#ifndef ZEDIT_TS_H
#define ZEDIT_TS_H

#include <stddef.h>
#include <stdint.h>
#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * zedit_ts.h — tsbundler C interface as defined by docs/zedit-spec.md.
 *
 * Must be included after zedit.h (for zedit_char_t, zedit_attr_t, etc.) or
 * define ZEDIT_STANDALONE to get minimal type definitions for testing.
 */

/* zedit_char_t: UCS-2 character (2 bytes on all platforms for this API).
 * zedit_attr_t: per-character syntax attribute code (16-bit).
 * zedit_rgb_t:  0x00RRGGBB packed colour.
 * zedit_vdu_attr: text-decoration flags. */
#ifndef ZEDIT_TYPES_DEFINED
#define ZEDIT_TYPES_DEFINED
typedef unsigned short zedit_char_t;
typedef unsigned short zedit_attr_t;
typedef unsigned int   zedit_rgb_t;
typedef unsigned int   zedit_vdu_attr;
#endif

#define ZEDIT_RGBNONE  0xFFFFFFFFU

#define ZEDIT_VDU_ATTR_NORMAL        0x00
#define ZEDIT_VDU_ATTR_BOLD          0x01
#define ZEDIT_VDU_ATTR_ITALIC        0x02
#define ZEDIT_VDU_ATTR_UNDERLINE     0x04
#define ZEDIT_VDU_ATTR_STRIKETHROUGH 0x08

/* One syntax-highlight element in the active theme. */
typedef struct {
    const char*   name;    /* Capture tag, e.g. "@keyword" */
    zedit_rgb_t   fg;      /* Foreground colour; ZEDIT_RGBNONE = use default */
    zedit_rgb_t   bg;      /* Background colour; ZEDIT_RGBNONE = use default */
    zedit_vdu_attr attrs;  /* Combination of ZEDIT_VDU_ATTR_* flags */
    int           index;   /* The zedit_attr_t value emitted in attrs[] output */
} zedit_syn_element;

/* tsbundler context and document handles */
typedef struct tsbundler_ctx_s *tsbundler_ctx;
typedef struct tsbundler_doc_s *tsbundler_doc;

/* Error codes */
typedef enum {
    TSBUNDLER_OK            = 0,
    TSBUNDLER_ERR_NOMEM     = 1,
    TSBUNDLER_ERR_NOLANG    = 2,
    TSBUNDLER_ERR_NOTHEME   = 3,
    TSBUNDLER_ERR_PARSE     = 4,
    TSBUNDLER_ERR_INTERNAL  = 99
} tsbundler_error;

/* Log types */
typedef enum {
    TSBUNDLER_LOG_PARSE = 0,
    TSBUNDLER_LOG_LEX   = 1,
    TSBUNDLER_LOG_INFO  = 2
} tsbundler_log_type;

/* Log callback */
typedef void (*tsbundler_log_fn)(void *payload, tsbundler_log_type log_type,
                                  const char *message);

/* Language enumeration callback */
typedef void (*tsbundler_lang_fn)(void *userdata, const char *name,
                                   const char **exts, int nexts);

/* Theme name enumeration callback */
typedef void (*tsbundler_theme_fn)(void *userdata, const char *name);

/* Theme element callback */
typedef void (*tsbundler_elem_fn)(void *userdata, const char *name,
                                   zedit_rgb_t fg, zedit_rgb_t bg,
                                   zedit_vdu_attr attrs, int index);

/* Text read callback (TSInputRead-compatible).
 * Returns pointer to raw bytes at (byte_offset, row, col); sets *bytes_read.
 * For UTF-16LE: col = char_idx * sizeof(zedit_char_t). */
typedef const char *(*tsbundler_read_fn)(void *payload, uint32_t byte_offset,
                                          uint32_t row, uint32_t col,
                                          uint32_t *bytes_read);

/* Attribute output callback.
 * Called before writing attrs for line `row`.  zedit returns a pointer to its
 * own zedit_attr_t[nchars] buffer, or NULL to skip that line. */
typedef zedit_attr_t *(*tsbundler_attrs_fn)(void *payload, uint32_t row,
                                             uint32_t nchars);

/* Text encoding */
typedef enum {
    TSBUNDLER_ENC_UTF8    = 0,
    TSBUNDLER_ENC_UTF16LE = 1,
    TSBUNDLER_ENC_UTF16BE = 2
} tsbundler_encoding;

/* Edit descriptor (all offsets in bytes; for UTF-16LE multiply char indices by 2) */
typedef struct {
    uint32_t start_byte;
    uint32_t old_end_byte;
    uint32_t new_end_byte;
    uint32_t start_row;
    uint32_t start_col;
    uint32_t old_end_row;
    uint32_t old_end_col;
    uint32_t new_end_row;
    uint32_t new_end_col;
} tsbundler_edit;

/* Changed-range callback for incremental edit */
typedef void (*tsbundler_range_fn)(void *userdata, uint32_t start_line,
                                    uint32_t end_line);

/* === Library context API (§4, §5) === */

tsbundler_ctx tsbundler_init(void);
void          tsbundler_free(tsbundler_ctx ctx);
void          tsbundler_set_logger(tsbundler_ctx ctx, tsbundler_log_fn logger,
                                    void *payload);

/* === Language enumeration (§6) === */

/* Returns total count.  Pass callback=NULL to count without firing callbacks. */
ssize_t tsbundler_enum_langs(tsbundler_ctx ctx, tsbundler_lang_fn callback,
                               void *userdata);
int     tsbundler_supports_lang(tsbundler_ctx ctx, const char *lang);

/* === Theme API (§7) === */

/* Returns total count.  Pass callback=NULL to count without firing callbacks. */
ssize_t tsbundler_enum_themes(tsbundler_ctx ctx, tsbundler_theme_fn callback,
                                void *userdata);

/* Activate theme; invokes callback once per element.
 * Pass callback=NULL to get element count without callbacks.
 * Returns element count on success, negative error code on failure. */
ssize_t tsbundler_theme_activate(tsbundler_ctx ctx, const char *name,
                                   tsbundler_elem_fn callback, void *userdata);

/* === Document API (§8) === */

tsbundler_doc tsbundler_doc_create(tsbundler_ctx ctx, const char *lang);
void          tsbundler_doc_free(tsbundler_doc doc);

tsbundler_error tsbundler_doc_parse_full(tsbundler_doc doc,
                                          tsbundler_read_fn read,
                                          void *read_payload,
                                          tsbundler_encoding encoding,
                                          uint32_t total_lines,
                                          tsbundler_attrs_fn attrs,
                                          void *attrs_payload);

tsbundler_error tsbundler_doc_edit(tsbundler_doc doc,
                                    const tsbundler_edit *edits, int nedits,
                                    tsbundler_read_fn read, void *read_payload,
                                    tsbundler_encoding encoding,
                                    uint32_t total_lines,
                                    tsbundler_attrs_fn attrs,
                                    void *attrs_payload,
                                    tsbundler_range_fn on_changed,
                                    void *on_changed_payload);

#ifdef __cplusplus
}
#endif

#endif /* ZEDIT_TS_H */
