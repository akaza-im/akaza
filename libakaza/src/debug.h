#ifndef LIBAKAZA_DEBUG_H_
#define LIBAKAZA_DEBUG_H_

#include <iostream>
#include <string>

static void dump_string(const std::string &buf) {
    const char *q = buf.c_str();
    for (std::size_t i = 0; i < buf.size(); i++) {
        std::cout << +((uint8_t) q[i]) << " ";
    }
    std::cout << std::endl;
}

#endif // LIBAKAZA_DEBUG_H_
