CXX=clang++
CXXOPTS=-W -Wno-unused-parameter -Wno-unused-variable -g -std=c++17
# Note: `shell` is a gnu make extension.
EXT_SUFFIX:=$(shell python3-config --extension-suffix)

all: pyakaza/bind$(EXT_SUFFIX)

pyakaza/bind$(EXT_SUFFIX): src/bind.cc
	$(CXX) -O3 $(CXXOPTS) -shared -lakaza -lmarisa -fPIC `python3 -m pybind11 \
		--includes` src/bind.cc \
		-o pyakaza/bind$(EXT_SUFFIX)

test: all
	pytest tests

clean:
	rm pyakaza/bind$(EXT_SUFFIX)

.PHONY: all test clean
