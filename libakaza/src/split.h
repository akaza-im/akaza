#ifndef LIBAKAZA_SPLIT_H
#define LIBAKAZA_SPLIT_H

#include <string>
#include <tuple>

static inline std::tuple<std::string, std::string> split2(const std::string &str, char sep, bool &splitted) {
    size_t pos = str.find_first_of(sep);
    if (pos == std::string::npos) {
        splitted = false;
        return std::make_tuple("", "");
    }
    return std::make_tuple(str.substr(0, pos), str.substr(pos + 1));
}

#endif //LIBAKAZA_SPLIT_H
