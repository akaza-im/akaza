#include "../include/akaza.h"
#include <regex>
#include <algorithm>
#include <cctype>
#include <string>
#include <codecvt>
#include "debug_log.h"

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

akaza::RomkanConverter::RomkanConverter(const std::map<std::string, std::string> &additional) {
    // romaji -> hiragana
    for (const auto &[rom, hira]: DEFAULT_ROMKAN_H) {
        _map[rom] = hira;
    }
    for (const auto &[rom, hira]: additional) {
        _map[rom] = hira;
    }

    std::vector<std::string> keys;
    keys.reserve(_map.size());
    for (const auto &[k, v]: _map) {
        keys.push_back(k);
    }
    std::sort(keys.begin(), keys.end(), [](auto &a, auto &b) {
        return a.length() > b.length();
    });

    std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;

    {
        std::wstring pattern_str = L"^(";
        for (const auto &key: keys) {
            pattern_str += quotemeta(cnv.from_bytes(key));
            pattern_str += L"|";
        }
        pattern_str += L".)";
        D(std::cout << "PATTERN: " << pattern_str << std::endl);

        _pattern.assign(pattern_str);
    }

    {
        std::wstring last_char_pattern = L"(";
        for (const auto &key: keys) {
            last_char_pattern += quotemeta(cnv.from_bytes(key));
            last_char_pattern += L"|";
        }
        last_char_pattern += L".)$";

        _last_char_pattern.assign(last_char_pattern);
    }

    /*
            self.pattern = re.compile(
            '(' + "|".join(sorted([re.escape(x) for x in self.map.keys()], key=_len_cmp)) + ')'
        )
        print('(' + "|".join(sorted([re.escape(x) for x in self.map.keys()], key=_len_cmp)) + r'|.)$')
        self.last_char_pattern = re.compile(
            '(?:' + "|".join(sorted([re.escape(x) for x in self.map.keys()], key=_len_cmp)) + r'|.)$'
        )
    def to_hiragana(self, s: str) -> str:
        """
        Convert a Romaji (ローマ字) to a Hiragana (平仮名).
        """

        s = s.lower()
        s = _normalize_double_n(s)
        return self.pattern.sub(lambda x: self.map[x.group(1)], s)

    def remove_last_char(self, s: str) -> str:
        return self.last_char_pattern.sub('', s)

     */
}

std::wstring akaza::RomkanConverter::remove_last_char(const std::wstring &s) {
    return std::regex_replace(s, _last_char_pattern, L"");
}

//     s = re.sub("nn", "n'", s)
static void replaceAll(std::string &str, const std::string &from, const std::string &to) {
    if (from.empty())
        return;
    size_t start_pos = 0;
    while ((start_pos = str.find(from, start_pos)) != std::string::npos) {
        str.replace(start_pos, from.length(), to);
        start_pos += to.length(); // In case 'to' contains 'from', like replacing 'x' with 'yx'
    }
}

std::wstring akaza::RomkanConverter::to_hiragana(const std::string &ss) {
    std::string s = ss;
    std::transform(s.begin(), s.end(), s.begin(),
                   [](unsigned char c) { return std::tolower(c); });

    replaceAll(s, "nn", "n'");

    std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;
    std::wstring result;
    std::wstring ws = cnv.from_bytes(s);
    std::wsmatch sm;
    while (std::regex_search(ws, sm, _pattern)) {
        std::wstring p = sm.str(1);
        ws = ws.substr(p.size());
        D(std::cout << cnv.to_bytes(p) << std::endl);
        std::string sp = cnv.to_bytes(p);
        if (_map.count(sp) > 0) {
            result += cnv.from_bytes(_map[sp]);
        } else {
            result += p;
        }
    }
    return result;
}

