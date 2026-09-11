#ifndef MARKSTONE_H
#define MARKSTONE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
  MARKSTONE_OK                 = 0,
  MARKSTONE_ERR_NULL_ARGUMENT  = 1,
  MARKSTONE_ERR_INVALID_UTF8   = 2,
  MARKSTONE_ERR_INPUT_TOO_LARGE= 3,
  MARKSTONE_ERR_DEPTH_EXCEEDED = 4,
  MARKSTONE_ERR_INTERNAL       = 5
} markstone_status;

/* --- Generic family: pure CommonMark + GFM --- */

/* The input need not be NUL-terminated; the length is passed explicitly.
   On success *out points at a NUL-terminated UTF-8 buffer and *out_len is
   its byte length excluding the NUL. On error *out and *out_len are left
   untouched. The returned buffer is released with markstone_free. */
markstone_status markstone_to_html(const char *input, size_t input_len,
                                   char **out, size_t *out_len);

markstone_status markstone_to_ast(const char *input, size_t input_len,
                                  char **out, size_t *out_len);

/* --- Actos family: the generic pipeline plus the mention/tag pass --- */

markstone_status markstone_actos_to_html(const char *input, size_t input_len,
                                         char **out, size_t *out_len);

markstone_status markstone_actos_to_ast(const char *input, size_t input_len,
                                        char **out, size_t *out_len);

/* --- Shared --- */

/* Releases a buffer returned by markstone_to_*. Does nothing if ptr is NULL.
   len must be exactly the out_len that call handed back. */
void markstone_free(char *ptr, size_t len);

/* Static semver string, never freed. */
const char *markstone_version(void);

/* AST JSON schema version. Increments when the schema breaks. */
uint32_t markstone_ast_schema_version(void);

#ifdef __cplusplus
}
#endif

#endif /* MARKSTONE_H */
