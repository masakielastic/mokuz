#include <php_embed.h>
#include <php_main.h>
#include <Zend/zend_string.h>

int run_php_script(const char* filename) {
    zend_file_handle file_handle;

    if (php_embed_init(0, NULL) == FAILURE) {
        return 1;
    }

    zend_string *fname = zend_string_init(filename, strlen(filename), 0);

    file_handle.type = ZEND_HANDLE_FILENAME;
    file_handle.filename = fname;
    file_handle.opened_path = NULL;
    file_handle.handle.fp = NULL;
    file_handle.buf = NULL;
    file_handle.primary_script = 1;

    zend_execute_scripts(ZEND_REQUIRE, NULL, 1, &file_handle);

    php_embed_shutdown();
    return 0;
}
