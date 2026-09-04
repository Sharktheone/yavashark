#include <stddef.h>

#if defined(_MSC_VER)
#include <malloc.h>
#define YS_ALLOCA(size) _alloca(size)
#elif defined(__GNUC__) || defined(__clang__)
#define YS_ALLOCA(size) __builtin_alloca(size)
#else
#error Unsupported compiler for the alloca helper
#endif

void yavashark_with_alloca(size_t size,
                          void (*callback)(void *, void *),
                          void *context) {
    void *buffer = YS_ALLOCA(size);
    callback(buffer, context);
}
