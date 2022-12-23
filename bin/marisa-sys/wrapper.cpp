#include "./wrapper.hpp"

// wrapper functions for inline methods.
// bindgen can't handle inline methods...

const marisa::Key * marisa_agent_key(marisa::Agent * agent) {
    return &agent->key();
}

const char* marisa_key_ptr(const marisa::Key * key) {
    return key->ptr();
}
