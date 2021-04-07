#include "solver.h"

#include <array>
#include <algorithm>
#include <cassert>
#include <cstring>

static const int SIZE=81;
static const int ROOT=9;

namespace
{
struct Grid
{
	using Board = std::array<int, SIZE>;
	Board board;

	bool solve();
private:
	bool solve(int pos);
	void fill(int pos, int num);
	bool isValidBoard() const;
	bool isValidPos(int pos) const;
	bool isValidRow(int pos) const;
	bool isValidCol(int pos) const;
	bool isValidSquare(int pos) const;
};

bool Grid::solve()
{
	return isValidBoard() && solve(0);
}

bool Grid::isValidBoard() const
{
	for (auto pos=0; pos < SIZE; ++pos)
	{
		if (board[pos] && !isValidPos(pos))
		{
			return false;
		}
	}
	return true;
}

bool Grid::solve(int pos)
{
	if (pos == SIZE)
	{
		// reached the end - assume solved
		return true;
	}

	if (board[pos])
	{
		return solve(pos+1);
	}

	auto copy{*this};

	for (auto i=1; i < ROOT+1; ++i)
	{
		copy.fill(pos, i);
		if (copy.isValidPos(pos) && copy.solve(pos+1))
		{
			// propagate solved board up
			board=copy.board;
			return true;
		}
	}
	return false;
}

void Grid::fill(int pos, int i)
{
	board[pos] = i;
}

bool Grid::isValidPos(int pos) const
{
	return isValidRow(pos) && isValidCol(pos) && isValidSquare(pos);
}

bool Grid::isValidRow(int pos) const
{
	auto row = pos/ROOT;
	auto firstInRow = row*ROOT;
	for (auto i = firstInRow; i < firstInRow+ROOT; ++i)
	{
		if (i != pos && board[i] == board[pos])
		{
			return false;
		}
	}
	return true;
}

bool Grid::isValidCol(int pos) const
{
	auto col = pos%ROOT;
	for (auto i = col; i < SIZE; i=i+ROOT)
	{
		if (i != pos && board[i] == board[pos])
		{
			return false;
		}
	}
	return true;
}

bool Grid::isValidSquare(int pos) const
{
	auto col = pos % ROOT;
	auto row = pos / ROOT;
	auto firstPosInSquare = ROOT * (row - (row % 3)) + (col - (col % 3)); 

	std::array<int, ROOT> positions{0, 1, 2, ROOT, ROOT+1, ROOT+2, 2*ROOT+1, 2*ROOT+2};
	for (auto squared : positions)
	{
		auto index = squared + firstPosInSquare;
		if (index != pos && board[index] == board[pos])
		{
			return false;
		}
	}
	return true;
}
} // unnamed namespace

extern "C"
{
	bool solve(int* data, int size)
	{
		assert(size == SIZE);
		Grid g;
		memcpy(&g.board, data, sizeof(int) * size);
		auto ans = g.solve();
		memcpy(data, &g.board, sizeof(int) * size);
		return ans;
	}
}

