#include <mysharedptr.h>
#include <cassert>

namespace mysharedptr
{
namespace {

void test_empty() {
	MySharedPtr<int> a;
	assert(a.use_count() == 0);
	assert(!a);
	assert(!a.get());
}

void test_reset() {
	MySharedPtr<int> a;
	a.reset(new int(4));
	assert(*a == 4);
	assert(a.use_count() == 1);
	a.reset(new int(3));
	assert(*a == 3);
	assert(a.use_count() == 1);
}

void test_reset_copy() {
	MySharedPtr<int> a(new int(3));
	auto b = a;
	assert(a == b); 
	assert(a.use_count() == 2);
	assert(b.use_count() == 2);

	b.reset(nullptr);
	assert(a.use_count() == 1);
	assert(b.use_count() == 0);

}

void test_assignment() {
	MySharedPtr<int> a;
	auto b = a;
	assert(a == b); 
	a.reset(new int(3));
	assert(a.use_count() == 1);
	assert(b.use_count() == 0);

	b = a;
	assert(a.use_count() == 2);
	assert(b.use_count() == 2);

	assert(b.get() == a.get());
}

void test_make_shared() {
	auto a = make_shared<int>(3);
	auto b = a;
	assert(a == b); 
	a.reset(new int(3));
	assert(a.use_count() == 1);
	assert(b.use_count() == 1);

	a = b;
	assert(a.use_count() == 2);
	assert(b.use_count() == 2);

	assert(b.get() == a.get());
	assert(*b.get() == *a.get());
}

}

}

int main() {
	using namespace mysharedptr;
	test_empty();
	test_reset();
	test_reset_copy();
	test_assignment();
	test_make_shared();
}

