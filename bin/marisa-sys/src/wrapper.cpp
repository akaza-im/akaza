#include "../wrapper.hpp"

// wrapper functions for inline methods.
// bindgen can't handle inline methods...

const marisa::Key * marisa_agent_key(marisa::Agent * agent) {
    return &agent->key();
}

const char* marisa_key_ptr(const marisa::Key * key) {
    return key->ptr();
}

uint32_t marisa_key_length(const marisa::Key * key) {
    return key->length();
}

size_t marisa_query_length(const marisa::Query * query) {
    return query->length();
}
const marisa::Query& marisa_agent_query(const marisa::Agent * agent) {
    return agent->query();
}
