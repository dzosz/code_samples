.PHONY: test all
all: test



DBG=-O0 -g
REL=-O3 -DNDEBUG
tests: tests.cpp firewall.hpp
	@echo "long compilation due to header only catch2 testing framework"
	@echo ""
	g++ -std=c++17 tests.cpp -Wall -Wno-unused-parameter -Wextra -o tests -march=native $(REL)


test: tests
	./tests
