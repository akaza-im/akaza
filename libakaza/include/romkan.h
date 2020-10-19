#ifndef LIBAKAZA_ROMKAN_H
#define LIBAKAZA_ROMKAN_H

#include <regex>
#include <unordered_map>

namespace akaza {
    class RomkanConverter {
    private:
        const std::wregex pattern_;
        const std::wregex last_char_pattern_;
        const std::unordered_map<std::wstring, std::wstring> map_;
    public:
        RomkanConverter(const std::unordered_map<std::wstring, std::wstring> &map,
                        const std::wregex &pattern,
                        const std::wregex &last_char_pattern) :
                map_(map),
                pattern_(pattern),
                last_char_pattern_(last_char_pattern) {}

        std::wstring remove_last_char(const std::wstring &s);

        std::wstring to_hiragana(const std::wstring &s);
    };

    std::shared_ptr<RomkanConverter> build_romkan_converter(const std::map<std::wstring, std::wstring> &additional);
}

#endif //LIBAKAZA_ROMKAN_H
