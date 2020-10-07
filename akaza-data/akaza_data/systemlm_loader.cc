#include <pybind11/pybind11.h>
#include <pybind11/stl.h>

#include <akaza/akaza.h>

namespace py = pybind11;

PYBIND11_MODULE(systemlm_loader, m) {
    m.doc() = "system lm"; // optional module docstring

    py::class_<akaza::SystemUnigramLM>(m, "SystemUnigramLM")
        .def(py::init())
        .def("load", &akaza::SystemUnigramLM::load)
        .def("find_unigram", &akaza::SystemUnigramLM::find_unigram)
        ;

    py::class_<akaza::SystemBigramLM>(m, "SystemBigramLM")
        .def(py::init())
        .def("load", &akaza::SystemBigramLM::load)
        .def("find_bigram", &akaza::SystemBigramLM::find_bigram)
        ;

    py::class_<akaza::BinaryDict>(m, "BinaryDict")
        .def(py::init())
        .def("load", &akaza::BinaryDict::load)
        .def("save", &akaza::BinaryDict::save)
        .def("build", &akaza::BinaryDict::build)
        .def("build_by_keyset", &akaza::BinaryDict::build_by_keyset)
        .def("find_kanjis", &akaza::BinaryDict::find_kanjis)
        .def("prefixes", &akaza::BinaryDict::prefixes)
        ;

    py::class_<akaza::tinylisp::TinyLisp>(m, "TinyLisp")
        .def(py::init())
        .def("run", &akaza::tinylisp::TinyLisp::run)
        ;

    py::class_<akaza::Node>(m, "Node")
        .def(py::init<size_t, const std::string&, const std::string&>())
        .def("get_key", &akaza::Node::get_key)
        .def("is_bos", &akaza::Node::is_bos)
        .def("is_eos", &akaza::Node::is_eos)
        .def("surface", &akaza::Node::surface)
        .def("get_yomi", &akaza::Node::get_yomi)
        .def("get_word", &akaza::Node::get_word)
        .def("get_cost", &akaza::Node::get_cost)
        .def("set_cost", &akaza::Node::set_cost)
        .def("get_start_pos", &akaza::Node::get_start_pos)
        .def("get_prev", &akaza::Node::get_prev)
        .def("set_prev", &akaza::Node::set_prev)
        .def("calc_node_cost", &akaza::Node::calc_node_cost)
        .def("get_bigram_cost", &akaza::Node::get_bigram_cost)
        .def("get_word_id", &akaza::Node::get_word_id)
        .def("create_bos", &akaza::Node::create_bos)
        .def("create_eos", &akaza::Node::create_eos)
        ;

    py::class_<akaza::UserLanguageModel>(m, "UserLanguageModel")
        .def(py::init<const std::string&, const std::string&>())
        .def("load_unigram", &akaza::UserLanguageModel::load_unigram)
        .def("load_bigram", &akaza::UserLanguageModel::load_bigram)
        .def("add_entry", &akaza::UserLanguageModel::add_entry)
        .def("get_unigram_cost", &akaza::UserLanguageModel::get_unigram_cost)
        .def("has_unigram_cost_by_yomi", &akaza::UserLanguageModel::has_unigram_cost_by_yomi)
        .def("get_bigram_cost", &akaza::UserLanguageModel::get_bigram_cost)
        .def("save", &akaza::UserLanguageModel::save)
        .def("should_save", &akaza::UserLanguageModel::should_save)
        ;
}
