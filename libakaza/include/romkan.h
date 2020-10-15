#ifndef LIBAKAZA_ROMKAN_H
#define LIBAKAZA_ROMKAN_H

#include <regex>

namespace akaza {
    class RomkanConverter {
    private:
        std::wregex _pattern;
        std::wregex _last_char_pattern;
        std::map<std::string, std::string> _map;
    public:
        RomkanConverter(const std::map<std::string, std::string> &additional);
        std::wstring remove_last_char(const std::wstring & s);
        std::wstring to_hiragana(const std::string & s);
    };
}

#endif //LIBAKAZA_ROMKAN_H
