#include <pybind11/pybind11.h>
#include <pybind11/stl.h>

#include <akaza/akaza.h>
#include <codecvt>

namespace py = pybind11;

PYBIND11_MODULE(bind, m) {
    m.doc() = "pyakaza"; // optional module docstring

    py::class_<akaza::Akaza, std::shared_ptr<akaza::Akaza>>(m, "Akaza")
            .def(py::init<std::shared_ptr<akaza::GraphResolver> &,
                    std::shared_ptr<akaza::RomkanConverter> &>())
            .def("convert", &akaza::Akaza::convert)
            .def("get_version", &akaza::Akaza::get_version);

    py::class_<akaza::RomkanConverter, std::shared_ptr<akaza::RomkanConverter>>(m, "RomkanConverter")
            .def(py::init<const std::unordered_map<std::wstring, std::wstring> &, const std::wregex &, const std::wregex &>())
            .def("to_hiragana", &akaza::RomkanConverter::to_hiragana)
            .def("remove_last_char", &akaza::RomkanConverter::remove_last_char);

    m.def("build_romkan_converter", &akaza::build_romkan_converter, "Build romaji to kana converter");

    py::class_<akaza::SystemUnigramLM, std::shared_ptr<akaza::SystemUnigramLM>>(m, "SystemUnigramLM")
            .def(py::init())
            .def("load", &akaza::SystemUnigramLM::load)
            .def("find_unigram", &akaza::SystemUnigramLM::find_unigram);

    py::class_<akaza::SystemBigramLM, std::shared_ptr<akaza::SystemBigramLM>>(m, "SystemBigramLM")
            .def(py::init())
            .def("load", &akaza::SystemBigramLM::load)
            .def("find_bigram", &akaza::SystemBigramLM::find_bigram);

    py::class_<akaza::BinaryDict, std::shared_ptr<akaza::BinaryDict>>(m, "BinaryDict")
            .def(py::init())
            .def("load", &akaza::BinaryDict::load)
            .def("save", &akaza::BinaryDict::save)
            .def("build", &akaza::BinaryDict::build)
            .def("build_by_keyset", &akaza::BinaryDict::build_by_keyset)
            .def("find_kanjis", &akaza::BinaryDict::find_kanjis);

    py::class_<akaza::tinylisp::TinyLisp, std::shared_ptr<akaza::tinylisp::TinyLisp>>(m, "TinyLisp")
            .def(py::init())
            .def("run", &akaza::tinylisp::TinyLisp::run);

    py::class_<akaza::Graph, std::shared_ptr<akaza::Graph>>(m, "Graph")
            .def(py::init())
            .def("dump", &akaza::Graph::dump)
            .def("get_items", &akaza::Graph::get_items);;

    py::class_<akaza::Node, std::shared_ptr<akaza::Node>>(m, "Node")
            .def(py::init<int, const std::wstring &, const std::wstring &, const std::wstring &,
                    bool, bool, int32_t, float>())
            .def("__eq__", &akaza::Node::operator==, py::is_operator())
            .def("get_key", &akaza::Node::get_key)
            .def("is_bos", &akaza::Node::is_bos)
            .def("is_eos", &akaza::Node::is_eos)
            .def("surface", &akaza::Node::surface)
            .def("get_yomi", &akaza::Node::get_yomi)
            .def("get_word", &akaza::Node::get_word)
            .def("get_start_pos", &akaza::Node::get_start_pos)
            .def("get_prev", &akaza::Node::get_prev)
            .def("calc_node_cost", &akaza::Node::calc_node_cost)
            .def("get_bigram_cost", &akaza::Node::get_bigram_cost)
            .def("get_word_id", &akaza::Node::get_word_id)
            .def("__repr__",
                 [](const akaza::Node &node) {
                     std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> cnv;
                     return "<akaza::Node yomi= '" + cnv.to_bytes(node.get_yomi()) + " word=" +
                            cnv.to_bytes(node.get_word()) + "'>";
                 }
            );

    py::class_<akaza::GraphResolver, std::shared_ptr<akaza::GraphResolver>>(m, "GraphResolver")
            .def(py::init<const std::shared_ptr<akaza::UserLanguageModel> &,
                    const std::shared_ptr<akaza::SystemUnigramLM> &,
                    const std::shared_ptr<akaza::SystemBigramLM> &,
                    const std::vector<std::shared_ptr<akaza::BinaryDict>> &,
                    const std::vector<std::shared_ptr<akaza::BinaryDict>> &>())
            .def("graph_construct", &akaza::GraphResolver::graph_construct)
            .def("fill_cost", &akaza::GraphResolver::fill_cost)
            .def("find_nbest", &akaza::GraphResolver::find_nbest);

    py::class_<akaza::UserLanguageModel, std::shared_ptr<akaza::UserLanguageModel>>(m, "UserLanguageModel")
            .def(py::init<const std::string &, const std::string &>())
            .def("load_unigram", &akaza::UserLanguageModel::load_unigram)
            .def("load_bigram", &akaza::UserLanguageModel::load_bigram)
            .def("add_entry", &akaza::UserLanguageModel::add_entry)
            .def("get_unigram_cost", &akaza::UserLanguageModel::get_unigram_cost)
            .def("has_unigram_cost_by_yomi", &akaza::UserLanguageModel::has_unigram_cost_by_yomi)
            .def("get_bigram_cost", &akaza::UserLanguageModel::get_bigram_cost)
            .def("save", &akaza::UserLanguageModel::save)
            .def("should_save", &akaza::UserLanguageModel::should_save);

    py::class_<akaza::Slice, std::shared_ptr<akaza::Slice>>(m, "Slice")
            .def(py::init<size_t, size_t>())
            .def("__repr__", &akaza::Slice::repr);
}
