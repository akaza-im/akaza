#include "wrapper.hpp"

marisa_obj * marisa_new() {
    marisa_obj* self = new marisa_obj();
    self->trie = new marisa::Trie();
    return self;
}

void marisa_release(marisa_obj* self) {
    delete self->trie;
    delete self;
}

void marisa_build(marisa_obj* self, marisa_keyset* keyset) {
    self->trie->build(*(keyset->keyset));
}

void marisa_load(marisa_obj* self, const char* filename) {
    self->trie->load(filename);
}

void marisa_save(marisa_obj* self, const char* filename) {
    self->trie->save(filename);
}

size_t marisa_num_keys(marisa_obj* self) {
    return self->trie->num_keys();
}

void marisa_predictive_search(marisa_obj *self, const char* query, size_t query_len, void* user_data, marisa_callback cb) {
    marisa::Agent agent;
    agent.set_query(query, query_len);

    while (self->trie->predictive_search(agent)) {
        if (!cb(user_data, agent.key().ptr(), agent.key().length(), agent.key().id())) {
            break;
        }
    }
}

// -----------------------------------------
// keyset
// -----------------------------------------

marisa_keyset* marisa_keyset_new() {
    marisa_keyset* self = new marisa_keyset();
    self->keyset = new marisa::Keyset();
    return self;
}

void marisa_keyset_push_back(marisa_keyset* self, const char* ptr, size_t length) {
    self->keyset->push_back(ptr, length);
}

void marisa_keyset_release(marisa_keyset* self) {
    delete self->keyset;
    delete self;
}

