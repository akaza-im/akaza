#ifndef LIBAKAZA_AKAZA_H_
#define LIBAKAZA_AKAZA_H_

#include "binary_dict.h"
#include "system_lm.h"
#include "tinylisp.h"
#include "user_language_model.h"
#include "node.h"
#include "graph.h"
#include "graph_resolver.h"
#include "romkan.h"

namespace akaza {
    class Akaza {
    private:
        GraphResolver *_graphResolver;
        RomkanConverter *_romkanConverter;
    public:
        Akaza(GraphResolver *graphResolver, RomkanConverter *romkanConverter) {
            _graphResolver = graphResolver;
            _romkanConverter = romkanConverter;
        }

        std::vector<std::vector<std::shared_ptr<Node>>> convert(
                const std::string &s,
                const std::optional<std::vector<Slice>>& forceSelectedClauses);
    };
}

#endif // LIBAKAZA_AKAZA_H_
