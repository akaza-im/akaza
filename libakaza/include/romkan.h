#ifndef LIBAKAZA_ROMKAN_H
#define LIBAKAZA_ROMKAN_H

#include <regex>

namespace akaza {
    class RomkanConverter {
    private:
        std::wregex pattern_;
        std::wregex last_char_pattern_;
        std::map<std::wstring, std::wstring> map_;
    public:
        RomkanConverter(const std::map<std::wstring, std::wstring> &additional);
        std::wstring remove_last_char(const std::wstring & s);
        std::wstring to_hiragana(const std::wstring & s);
    };

    // TODO: implement build_romkan_converter(additional)
}

#endif //LIBAKAZA_ROMKAN_H
