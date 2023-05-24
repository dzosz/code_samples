all: test_mysharedptr test_myweakptr

test_mysharedptr: mysharedptr.h test_mysharedptr.cpp
	$(CXX) test_mysharedptr.cpp -g -o $@

test_myweakptr: mysharedptr.h  myweakptr.h test_myweakptr.cpp
	$(CXX) test_myweakptr.cpp -g -o $@
