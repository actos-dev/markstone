#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "markstone.h"

int main(void) {
    printf("Starting C integration tests for markstone ABI...\n");

    /* 1. Version & Schema Version */
    const char *ver = markstone_version();
    assert(ver != NULL);
    assert(strcmp(ver, "0.1.0") == 0);

    uint32_t schema_ver = markstone_ast_schema_version();
    assert(schema_ver == 1);

    /* 2. Generic HTML Conversion */
    const char input[] = "# Heading\n\nParagraph with **bold** text.\n\n@user and #tag\n";
    size_t input_len = strlen(input);
    char *out = NULL;
    size_t out_len = 0;

    markstone_status status = markstone_to_html(input, input_len, &out, &out_len);
    assert(status == MARKSTONE_OK);
    assert(out != NULL);
    assert(out_len > 0);
    assert(out[out_len] == '\0');
    assert(strlen(out) == out_len);
    assert(strstr(out, "<h1>Heading</h1>") != NULL);
    assert(strstr(out, "<strong>bold</strong>") != NULL);
    /* In generic mode, mention and tag are plain text */
    assert(strstr(out, "@user and #tag") != NULL);
    assert(strstr(out, "href=\"/u/user\"") == NULL);
    markstone_free(out, out_len);

    /* 3. Actos HTML Conversion */
    out = NULL;
    out_len = 0;
    status = markstone_actos_to_html(input, input_len, &out, &out_len);
    assert(status == MARKSTONE_OK);
    assert(out != NULL);
    assert(out_len > 0);
    assert(out[out_len] == '\0');
    assert(strstr(out, "<a href=\"/u/user\" class=\"mention\">@user</a>") != NULL);
    assert(strstr(out, "<a href=\"/t/tag\" class=\"tag\">#tag</a>") != NULL);
    markstone_free(out, out_len);

    /* 4. Generic AST Conversion */
    out = NULL;
    out_len = 0;
    status = markstone_to_ast(input, input_len, &out, &out_len);
    assert(status == MARKSTONE_OK);
    assert(out != NULL);
    assert(out_len > 0);
    assert(out[out_len] == '\0');
    assert(strstr(out, "\"schema\":1") != NULL);
    assert(strstr(out, "\"type\":\"document\"") != NULL);
    assert(strstr(out, "\"type\":\"mention\"") == NULL);
    markstone_free(out, out_len);

    /* 5. Actos AST Conversion */
    out = NULL;
    out_len = 0;
    status = markstone_actos_to_ast(input, input_len, &out, &out_len);
    assert(status == MARKSTONE_OK);
    assert(out != NULL);
    assert(out_len > 0);
    assert(out[out_len] == '\0');
    assert(strstr(out, "\"schema\":1") != NULL);
    assert(strstr(out, "\"type\":\"mention\"") != NULL);
    assert(strstr(out, "\"username\":\"user\"") != NULL);
    assert(strstr(out, "\"type\":\"tag\"") != NULL);
    assert(strstr(out, "\"name\":\"tag\"") != NULL);
    markstone_free(out, out_len);

    /* 6. Empty Input */
    out = NULL;
    out_len = 999;
    status = markstone_to_html("", 0, &out, &out_len);
    assert(status == MARKSTONE_OK);
    assert(out != NULL);
    assert(out_len == 0);
    assert(out[0] == '\0');
    markstone_free(out, out_len);

    /* 7. NULL Input with len 0 */
    out = NULL;
    out_len = 999;
    status = markstone_to_html(NULL, 0, &out, &out_len);
    assert(status == MARKSTONE_OK);
    assert(out != NULL);
    assert(out_len == 0);
    assert(out[0] == '\0');
    markstone_free(out, out_len);

    /* 8. Embedded NUL */
    const char embedded_nul[] = "Hello\0World";
    out = NULL;
    out_len = 0;
    status = markstone_to_html(embedded_nul, sizeof(embedded_nul) - 1, &out, &out_len);
    assert(status == MARKSTONE_OK);
    assert(out != NULL);
    assert(out_len > 0);
    assert(out[out_len] == '\0');
    markstone_free(out, out_len);

    /* 9. NULL Argument Handling */
    char *sentinel_ptr = (char *)(uintptr_t)0x12345678;
    size_t sentinel_len = (size_t)0xdeadbeef;
    out = sentinel_ptr;
    out_len = sentinel_len;

    status = markstone_to_html(input, input_len, NULL, &out_len);
    assert(status == MARKSTONE_ERR_NULL_ARGUMENT);
    assert(out_len == sentinel_len);

    status = markstone_to_html(input, input_len, &out, NULL);
    assert(status == MARKSTONE_ERR_NULL_ARGUMENT);
    assert(out == sentinel_ptr);

    status = markstone_to_html(NULL, 10, &out, &out_len);
    assert(status == MARKSTONE_ERR_NULL_ARGUMENT);
    assert(out == sentinel_ptr);
    assert(out_len == sentinel_len);

    /* 10. Invalid UTF-8 */
    const char invalid_utf8[] = "\xff\xfe\xfd";
    status = markstone_to_html(invalid_utf8, sizeof(invalid_utf8) - 1, &out, &out_len);
    assert(status == MARKSTONE_ERR_INVALID_UTF8);
    assert(out == sentinel_ptr);
    assert(out_len == sentinel_len);

    /* 11. markstone_free(NULL) */
    markstone_free(NULL, 0);
    markstone_free(NULL, 1024);

    printf("All C integration assertions passed successfully!\n");
    return 0;
}
