#include "../include/akaza.h"
#include <regex>
#include <algorithm>
#include <cctype>
#include <string>
#include <codecvt>
#include "debug_log.h"

#include "romkan_default.h"

static std::string quotemeta(const std::string &input) {
    std::regex specialChars{R"([-\[\]{}()*+?.,\^$|#\s])"};
    return std::regex_replace(input, specialChars, R"(\$&)");
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

    {
        std::string pattern_str = "^(";
        for (const auto &key: keys) {
            pattern_str += quotemeta(key);
            pattern_str += "|";
        }
        pattern_str += ".)";
        D(std::cout << "PATTERN: " << pattern_str << std::endl);

        std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;
        auto wpattern_str = cnv.from_bytes(pattern_str);

        _pattern.assign(wpattern_str);
    }

    {
        std::string last_char_pattern = "(";
        for (const auto &key: keys) {
            last_char_pattern += quotemeta(key);
            last_char_pattern += "|";
        }
        last_char_pattern += ".)$";

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

std::string akaza::RomkanConverter::remove_last_char(const std::string &s) {
    return std::regex_replace(s, _last_char_pattern, "");
}

static std::string normalize_double_n(const std::string &s) {
    //     s = re.sub("nn", "n'", s)
    return std::regex_replace(s, std::regex("nn"), "n'");

}

std::string akaza::RomkanConverter::to_hiragana(const std::string &ss) {
    std::string s = ss;
    std::transform(s.begin(), s.end(), s.begin(),
                   [](unsigned char c) { return std::tolower(c); });

    s = normalize_double_n(s);

    std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;
    std::string result;
    std::wstring ws = cnv.from_bytes(s);
    std::wsmatch sm;
    while (std::regex_search(ws, sm, _pattern)) {
        std::wstring p = sm.str(1);
        ws = ws.substr(p.size());
        D(std::cout << cnv.to_bytes(p) << std::endl);
        std::string sp = cnv.to_bytes(p);
        if (_map.count(sp) > 0) {
            result += _map[sp];
        } else {
            result += sp;
        }
    }
    return result;
}

