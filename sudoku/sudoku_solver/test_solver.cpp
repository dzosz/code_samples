#include "solver.h"

#include <vector>
#include <sstream>
#include <iostream>
#include <cmath>

int get_root(int n)
{
	return std::lround(sqrt(n));
}

bool is_perfect_square(int n) {
	if (n <= 0)
		return false;
	auto root{get_root(n)}; 

	return n == root * root;
}

std::string to_string(int* grid, int len) {
	auto root{get_root(len)}; 

	std::stringstream ss;

	for (int i =0; i < root; ++i)
	{
		for (int j =0; j < root; ++j)
		{
			ss << grid[i*root+j] << " ";
		}
		ss << '\n';
	}
	return ss.str();
}

int main(int argc, char** argv)
{
	if (argc < 2 || !is_perfect_square(argc-1))
	{
		std::cerr << "wrong number of args (" << argc-1 << ")" << std::endl;
		exit(1);
	}
	std::vector<int> numbers(argc-1);
	for (auto i = 1; i < argc; ++i)
	{
		std::stringstream num(argv[i]);
		num >> numbers[i-1];
	}
	auto res = solve(numbers.data(), numbers.size());
	if (res)
	{
		std::cout << "SOLVED" << std::endl;
		std::cout << to_string(numbers.data(), numbers.size()) << std::flush;
	}
	else
	{
		std::cout << "UNSOLVABLE" << std::endl;
	}
	exit(!res);
}
