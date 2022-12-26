#include <marisa.h>
#include <stddef.h>

#pragma once

typedef struct marisa_obj {
    marisa::Trie* trie;
} marisa_obj;

typedef struct marisa_keyset {
    marisa::Keyset* keyset;
} marisa_keyset;

typedef bool (*marisa_callback)(void* user_data, const char* key, size_t len, size_t id);

extern "C" {
    marisa_obj * marisa_new();
    void marisa_release(marisa_obj* self);
    void marisa_build(marisa_obj* self, marisa_keyset* keyset);
    void marisa_load(marisa_obj* self, const char* filename);
    void marisa_save(marisa_obj* self, const char* filename);
    void marisa_predictive_search(marisa_obj *self, const char* query, size_t query_len, void* user_data, marisa_callback cb);
    size_t marisa_num_keys(marisa_obj* self);

    marisa_keyset* marisa_keyset_new();
    void marisa_keyset_push_back(marisa_keyset* self, const char* ptr, size_t length);
    void marisa_keyset_release(marisa_keyset* self);
}

