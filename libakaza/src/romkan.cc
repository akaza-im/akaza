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

std::shared_ptr<akaza::RomkanConverter>
akaza::build_romkan_converter(const std::map<std::wstring, std::wstring> &additional) {
    std::unordered_map<std::wstring, std::wstring> map;

    // romaji -> hiragana
    for (const auto &[rom, hira]: DEFAULT_ROMKAN_H) {
        map[rom] = hira;
    }
    for (const auto &[rom, hira]: additional) {
        map[rom] = hira;
    }

    std::vector<std::wstring> keys;
    keys.reserve(map.size());
    for (const auto &[k, v]: map) {
        keys.push_back(k);
    }
    std::sort(keys.begin(), keys.end(), [](auto &a, auto &b) {
        return a.length() > b.length();
    });


    std::wregex pattern;
    std::wregex last_char_pattern;
    {
        std::wstring pattern_str = L"^(";
        for (const auto &key: keys) {
            pattern_str += quotemeta(key);
            pattern_str += L"|";
        }
        pattern_str += L".)";
        D(std::wcout << "PATTERN: " << pattern_str << std::endl);

        pattern.assign(pattern_str);
    }

    {
        std::wstring last_char_pattern_src = L"(";
        for (const auto &key: keys) {
            last_char_pattern_src += quotemeta(key);
            last_char_pattern_src += L"|";
        }
        last_char_pattern_src += L".)$";

        last_char_pattern.assign(last_char_pattern_src);
    }

    return std::make_shared<akaza::RomkanConverter>(map, pattern, last_char_pattern);
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
        auto search = map_.find(p);
        if (search != map_.cend()) {
            result += search->second;
        } else {
            result += p;
        }
    }
    return result;
}

