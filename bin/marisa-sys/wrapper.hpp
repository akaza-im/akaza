// wrapper
#include <marisa.h>

const marisa::Key * marisa_agent_key(marisa::Agent * agent);
const char* marisa_key_ptr(const marisa::Key * key);
uint32_t marisa_key_length(const marisa::Key * key);
size_t marisa_query_length(const marisa::Query * query);
const marisa::Query& marisa_agent_query(const marisa::Agent * agent);
