#ifndef LIBAKAZA_ROMKAN_H
#define LIBAKAZA_ROMKAN_H

#include <regex>

namespace akaza {
    class RomkanConverter {
    private:
        std::wregex pattern_;
        std::wregex last_char_pattern_;
        std::map<std::string, std::string> map_;
    public:
        RomkanConverter(const std::map<std::string, std::string> &additional);
        std::wstring remove_last_char(const std::wstring & s);
        std::wstring to_hiragana(const std::string & s);
    };
}

#endif //LIBAKAZA_ROMKAN_H
