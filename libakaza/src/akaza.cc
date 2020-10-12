#include "../include/akaza.h"
#include "debug_log.h"

std::vector<std::vector<std::shared_ptr<akaza::Node>>> akaza::Akaza::convert(
        const std::string &src,
        const std::optional<std::vector<akaza::Slice>>& forceSelectedClauses) {
    if (!src.empty() && isupper(src[0]) && !forceSelectedClauses.has_value()) {
        return {{std::make_shared<akaza::Node>(0, src, src)}};
    }

    std::string hiragana = _romkanConverter->to_hiragana(src);
    D(std::cout << "HIRAGANA=" << hiragana << std::endl);

    // 子音だが、N は NN だと「ん」になるので処理しない。
    std::regex trailing_consonant(R"(^(.*?)([qwrtypsdfghjklzxcvbm]+)$)");
    std::smatch sm;
    std::string consonant;
    if (std::regex_match(hiragana, sm, trailing_consonant)) {
        hiragana = sm.str(1);
        consonant = sm.str(2);
        D(std::cout << "CONSONANT=" << consonant << std::endl);
    }

    Graph graph = _graphResolver->graph_construct(hiragana, forceSelectedClauses);
    _graphResolver->fill_cost(graph);
    D(graph.dump());
    std::vector<std::vector<std::shared_ptr<akaza::Node>>> nodes = _graphResolver->find_nbest(graph);
    if (consonant.empty()) {
        return nodes;
    } else {
        nodes.push_back({{
                                 std::make_shared<akaza::Node>(
                                         src.size(),
                                         consonant,
                                         consonant
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
