#include "../include/romkan.h"
#include <regex>
#include <algorithm>
#include <cctype>
#include <string>
#include "debug_log.h"
#include <iostream>
#include "romkan_default.h"

static std::wstring quotemeta(const std::wstring &input) {
    std::wstring buffer;
    for (const auto &c: input) {
        switch (c) {
            case L'-':
            case L'[':
            case L']':
            case L'{':
            case L'}':
            case L'(':
            case L')':
            case L'*':
            case L'+':
            case L'?':
            case L'.':
            case L',':
            case L'\\':
            case L'^':
            case L'$':
            case L'|':
            case L'#':
            case L' ':
            case L'\t':
                buffer += L'\\';
                buffer += c;
                break;
            default:
                buffer += c;
        }
    }
    return buffer;
}

akaza::RomkanConverter::RomkanConverter(const std::map<std::wstring, std::wstring> &additional) {
    // romaji -> hiragana
    for (const auto &[rom, hira]: DEFAULT_ROMKAN_H) {
        map_[rom] = hira;
    }
    for (const auto &[rom, hira]: additional) {
        map_[rom] = hira;
    }

    std::vector<std::wstring> keys;
    keys.reserve(map_.size());
    for (const auto &[k, v]: map_) {
        keys.push_back(k);
    }
    std::sort(keys.begin(), keys.end(), [](auto &a, auto &b) {
        return a.length() > b.length();
    });


    {
        std::wstring pattern_str = L"^(";
        for (const auto &key: keys) {
            pattern_str += quotemeta(key);
            pattern_str += L"|";
        }
        pattern_str += L".)";
        D(std::wcout << "PATTERN: " << pattern_str << std::endl);

        pattern_.assign(pattern_str);
    }

    {
        std::wstring last_char_pattern = L"(";
        for (const auto &key: keys) {
            last_char_pattern += quotemeta(key);
            last_char_pattern += L"|";
        }
        last_char_pattern += L".)$";

        last_char_pattern_.assign(last_char_pattern);
    }

}

std::wstring akaza::RomkanConverter::remove_last_char(const std::wstring &s) {
    return std::regex_replace(s, last_char_pattern_, L"");
}

//     s = re.sub("nn", "n'", s)
static void replaceAll(std::wstring &str, const std::wstring &from, const std::wstring &to) {
    if (from.empty()) {
        return;
    }

    size_t start_pos = 0;
    while ((start_pos = str.find(from, start_pos)) != std::string::npos) {
        str.replace(start_pos, from.length(), to);
        start_pos += to.length(); // In case 'to' contains 'from', like replacing 'x' with 'yx'
    }
}

std::wstring akaza::RomkanConverter::to_hiragana(const std::wstring &ss) {
    std::wstring ws = ss;
    std::transform(ws.begin(), ws.end(), ws.begin(),
                   [](auto &c) { return std::tolower(c); });

    replaceAll(ws, L"nn", L"n'");

    std::wstring result;
    std::wsmatch sm;
    while (std::regex_search(ws, sm, pattern_)) {
        std::wstring p = sm.str(1);
        ws = ws.substr(p.size());
        D(std::wcout << p << std::endl);
        if (map_.count(p) > 0) {
            result += map_[p];
        } else {
            result += p;
        }
    }
    return result;
}

