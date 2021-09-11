#include "solver.h"

#include <array>
#include <algorithm>
#include <cassert>
#include <cstring>
#include <utility> // std::pair
#include <minisat/core/Solver.h>

static const int SIZE=81;
static const int ROOT=9;

namespace
{
struct Grid;

class SatSolver {
public:
    SatSolver(Grid& grid);
    bool solve();
private:
    void copy_solution();
    void one_square_one_value();
    void exactly_one_true(Minisat::vec<Minisat::Lit>& literals);
    void non_duplicated_values();

    Grid& grid;
    bool solvable = true;
    Minisat::Solver solver;
};


std::pair<int, int> getRowColumnFromSingleIndex(int index) {
	auto col = index % ROOT;
	auto row = index / ROOT;

    return {row, col};
}

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
	if (isValidBoard()) {
        auto brute = solve(0);
        return brute;
        //SAT solver is actually much slower on average test cases! 25ms instead of 3 ms brute force
        //auto sat = SatSolver(*this);
        //return sat.solve();
    }
    return false;

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
    auto [row, col] = getRowColumnFromSingleIndex(pos);
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

// MINISAT SOLVER
const int rows = ROOT;
const int columns = ROOT;
const int values = ROOT;

Minisat::Var to_minisat_var(int row, int column, int value) {
    return row * columns * values + column * values + value;
}

SatSolver::SatSolver(Grid& grid) : grid(grid) {
    for (int idx = 0; idx< SIZE*values; ++idx) {
        this->solver.newVar();
    }
    this->one_square_one_value();
    this->non_duplicated_values();

}

void SatSolver::non_duplicated_values() {
    // In each row, for each value, forbid two column sharing that value
    for (int row = 0; row< rows; ++row) {
        for (int value = 0; value< values; ++value) {
            Minisat::vec<Minisat::Lit> literals;
            for (int col = 0; col< columns; ++col) {
                literals.push(Minisat::mkLit(to_minisat_var(row, col, value)));
            }
            exactly_one_true(literals);
        }
    }
    // In each column, for each value, forbid two rows sharing that value
    for (int col = 0; col< columns; ++col) {
        for (int value = 0; value< values; ++value) {
            Minisat::vec<Minisat::Lit> literals;
            for (int row = 0; row< rows; ++row) {
                literals.push(Minisat::mkLit(to_minisat_var(row, col, value)));
            }
            exactly_one_true(literals);
        }
    }
    // Now forbid sharing in the 3x3 boxes
    for (int r =0; r < ROOT; r+= 3) {
        for (int c = 0; c < ROOT; c+= 3) {
            for (int value = 0; value < values; ++value) {
                Minisat::vec<Minisat::Lit> literals;
                for (int rr = 0; rr < 3 ; ++rr) {
                    for (int cc = 0; cc < 3; ++cc) {
                        literals.push(Minisat::mkLit(to_minisat_var(r + rr, c + cc, value)));
                    }
                }
                exactly_one_true(literals);
            }
        }
    }
}

void SatSolver::one_square_one_value() {
    for (int idx = 0; idx< SIZE; ++idx) {
        auto [row, col] = getRowColumnFromSingleIndex(idx);
        Minisat::vec<Minisat::Lit> literals;
        for (int value = 0; value < values; ++value) {
            literals.push(Minisat::mkLit(to_minisat_var(row, col, value)));
        }
        this->exactly_one_true(literals);
    }
}

void SatSolver::exactly_one_true(Minisat::vec<Minisat::Lit>& literals) {
    solver.addClause(literals);
    for (int i = 0; i< literals.size(); ++i) {
        for (int j = i+1; j< literals.size(); ++j) {
            solver.addClause(~literals[i], ~literals[j]);
        }
    }
}


bool SatSolver::solve() {
    for (int idx = 0; idx< SIZE; ++idx) {
        auto value = this->grid.board[idx];
        auto [row, col] = getRowColumnFromSingleIndex(idx);

        if (value) {
            this->solvable &= this->solver.addClause(Minisat::mkLit(to_minisat_var(row, col, value-1)));
        }
    }

    this->solvable = solvable && this->solver.solve();
    if (this->solvable) {
        this->copy_solution();
    }
    return this->solvable;
}

void SatSolver::copy_solution() {
    for (int idx = 0; idx< SIZE; ++idx) {
        auto [row, col] = getRowColumnFromSingleIndex(idx);

        int found = 0;
        for (int val = 0; val < values; ++val) {
            if (this->solver.modelValue(to_minisat_var(row, col, val)).isTrue()) {
                ++found;
                this->grid.board[idx] = val + 1;
            }
        }
        assert(found == 1 && "The SAT solver assigned one position more than one value");
    }
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
		if (ans) {
            memcpy(data, &g.board, sizeof(int) * size);
        }
		return ans;
	}
}

