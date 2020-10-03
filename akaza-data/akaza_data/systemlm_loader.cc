#include <pybind11/pybind11.h>
#include "../src/system_lm.h"

int add(int i, int j) {
    return i + j;
}

namespace py = pybind11;

PYBIND11_MODULE(systemlm_loader, m) {
    m.doc() = "system lm"; // optional module docstring

    m.def("add", &add, "A function which adds two numbers");

    py::class_<akaza::SystemLM>(m, "SystemLM")
        .def(py::init())
        .def("load", &akaza::SystemLM::load)
        .def("find_unigram", &akaza::SystemLM::find_unigram)
        .def("find_bigram", &akaza::SystemLM::find_bigram)
        ;
}
