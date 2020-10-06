#include <pybind11/pybind11.h>
#include <pybind11/stl.h>

#include "../src/akaza.h"

namespace py = pybind11;

PYBIND11_MODULE(systemlm_loader, m) {
    m.doc() = "system lm"; // optional module docstring

    py::class_<akaza::SystemLM>(m, "SystemLM")
        .def(py::init())
        .def("load", &akaza::SystemLM::load)
        .def("find_unigram", &akaza::SystemLM::find_unigram)
        .def("find_bigram", &akaza::SystemLM::find_bigram)
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
}
