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

    if (!src.empty() && my_isupper(src[0]) && !forceSelectedClauses.has_value()) {
        D(std::wcout << "Upper case" << src[0]
                     << " " << __FILE__ << ":" << __LINE__ << std::endl);
        return {{std::make_shared<akaza::Node>(0, src, src)}};
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
                                 std::make_shared<akaza::Node>(
                                         src.size(),
                                         cnv.from_bytes(consonant),
                                         cnv.from_bytes(consonant)
                                 )
                         }});
        return nodes;
    }

    /*
         if len(src) > 0 and src[0].isupper() and not force_selected_clause:
            # 最初の文字が大文字で、文節の強制指定がない場合、アルファベット強制入力とする。
            return [[
                Node(
                    start_pos=0,
                    word=src,
                    yomi=src,
                )
            ]]

        hiragana: str = self.romkan.to_hiragana(src)

        # 末尾の子音を変換対象外とする。
        m = TRAILING_CONSONANT_PATTERN.match(hiragana)
        if m:
            hiragana = m[1]
            consonant = m[2]
            print(f"{hiragana} {consonant}")

        katakana: str = jaconv.hira2kata(hiragana)
        self.logger.info(f"convert: src={src} hiragana={hiragana} katakana={katakana}")

        t0 = time.time()
        ht = dict(self.resolver.lookup(hiragana))
        graph = self.resolver.graph_construct(hiragana, ht, force_selected_clause)
        self.logger.info(
            f"graph_constructed: src={src} hiragana={hiragana} katakana={katakana}: {time.time() - t0} seconds")
        clauses = self.resolver.viterbi(graph)
        self.logger.info(
            f"converted: src={src} hiragana={hiragana} katakana={katakana}: {time.time() - t0} seconds")

        if m:
            clauses.append([Node(
                start_pos=len(src),
                word=consonant,
                yomi=consonant,
            )])
            return clauses
        else:
            return clauses

     */
}
