#include <codecvt>
#include "../include/akaza.h"
#include "debug_log.h"

std::string akaza::Akaza::get_version() {
    return "202010140940";
}

static inline bool my_isupper(wchar_t c) {
    return 'A' <= c && c <= 'Z';
}

std::vector<std::vector<std::shared_ptr<akaza::Node>>> akaza::Akaza::convert(
        const std::wstring &src,
        const std::optional<std::vector<akaza::Slice>> &forceSelectedClauses) {
    std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;

    D(std::wcout << "Akaza::convert '"
                 << src << "' (HASH="
                 << std::hash<std::wstring>{}(src)
                 << ")"
                 << " " << __FILE__ << ":" << __LINE__ << std::endl);
    assert(!forceSelectedClauses.has_value() || !forceSelectedClauses.value().empty());

    if (!src.empty() && my_isupper(src[0]) && !forceSelectedClauses.has_value()
        || src.rfind(L"https://", 0) == 0 || src.rfind(L"http://", 0) == 0) {
        D(std::wcout << "Upper case" << src[0]
                     << " " << __FILE__ << ":" << __LINE__ << std::endl);
        return {{akaza::create_node(graphResolver_->system_unigram_lm_, 0, src, src)}};
    }

    std::wstring whiragana = romkanConverter_->to_hiragana(src);
    std::string hiragana = cnv.to_bytes(whiragana);
    D(std::cout << "HIRAGANA=" << hiragana << std::endl);

    // 子音だが、N は NN だと「ん」になるので処理しない。
    std::string consonant;
    {
        std::wregex trailing_consonant(cnv.from_bytes(R"(^(.*?)([qwrtypsdfghjklzxcvbm]+)$)"));
        std::wsmatch sm;
        if (std::regex_match(whiragana, sm, trailing_consonant)) {
            hiragana = cnv.to_bytes(sm.str(1));
            consonant = cnv.to_bytes(sm.str(2));
            D(std::cout << "CONSONANT=" << consonant << std::endl);
        }
    }

    Graph graph = graphResolver_->graph_construct(cnv.from_bytes(hiragana), forceSelectedClauses);
    graphResolver_->fill_cost(graph);
    D(graph.dump());
    std::vector<std::vector<std::shared_ptr<akaza::Node>>> nodes = graphResolver_->find_nbest(graph);
    if (consonant.empty()) {
        return nodes;
    } else {
        D(std::cout << " Adding Consonant=" << consonant << std::endl);
        nodes.push_back({{
                                 akaza::create_node(
                                         graphResolver_->system_unigram_lm_,
                                         src.size(),
                                         cnv.from_bytes(consonant),
                                         cnv.from_bytes(consonant)
                                 )
                         }});
        return nodes;
    }
}
