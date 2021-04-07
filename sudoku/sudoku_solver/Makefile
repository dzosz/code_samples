CXXFLAGS = -Wall -Wpedantic -Wextra -Ofast -ggdb3 -std=c++17 -march=native -static-libstdc++ -static-libgcc -fno-exceptions -fno-rtti -DNDEBUG
LDFLAGS = -fPIC -Wl,-rpath '-Wl,$$ORIGIN'
solvable_test_cases := $(wildcard test_cases/solvable*)
unsolvable_test_cases := $(wildcard test_cases/unsolvable*)

.PHONY: $(solvable_test_cases) $(unsolvable_test_cases) test all clean install

all : libsolver.so test_solver

libsolver.so : solver.h solver.cpp
	g++ -shared solver.cpp -o $@ $(CXXFLAGS) ${LDFLAGS}

test_solver : libsolver.so  test_solver.cpp
	g++ test_solver.cpp -o $@ -L. -lsolver $(CXXFLAGS) $(LDFLAGS)

clean :
	rm -rf ./test_solver libsolver.so install/

test : test_solver $(unsolvable_test_cases) $(solvable_test_cases)

install : libsolver.so solver.h
	@mkdir -p install
	cp ./libsolver.so ./solver.h install/
	strip ./install/libsolver.so

$(solvable_test_cases): test_solver
		@echo $@
		@./test_solver $$(cat ./$@);
		@echo;

$(unsolvable_test_cases): test_solver
		@echo $@
		@! ./test_solver $$(cat ./$@);
		@echo;
