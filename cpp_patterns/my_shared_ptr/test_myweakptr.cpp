#include <mysharedptr.h>
#include <myweakptr.h>
#include <cassert>

namespace mysharedptr
{
namespace {

void test_empty() {
	MySharedPtr<int> a;
	MyWeakPtr<int> w(a);
	assert(w.expired());
	auto a2 = w.lock();
	assert(!a2);
}

void test_lock() {
	MySharedPtr<int> a(new int(3));
	MyWeakPtr<int> w(a);
	assert(!w.expired());
	{
		auto a2 = w.lock();
		assert(a.use_count() == 2);
		assert(a2.use_count() == 2);
		assert(w.use_count() == 2);
	}
	assert(a.use_count() == 1);
	assert(w.use_count() == 1);
}

void test_reset() {
	MySharedPtr<int> a(new int(3));
	MyWeakPtr<int> w(a);
	a.reset(nullptr);
	assert(a.use_count() == 0);
	assert(w.use_count() == 0);
	assert(w.weak_count() == 1);
	assert(w.expired());
}

void test_counter_values() {
	MySharedPtr<int> a(new int(3));
	MyWeakPtr<int> w(a);
	assert(w.weak_count() == 1);
	{
		MyWeakPtr<int> w2(a);
		assert(w2.weak_count() == 2);
	}
	assert(w.weak_count() == 1);
}

void test_empty_copy() {
	MyWeakPtr<int> w;
	assert(w.expired());
	auto w2 = w;
	assert(w2.expired());
	assert(w.weak_count() == 0);
	assert(w2.weak_count() == 0);
}

void test_copy() {
	MySharedPtr<int> a(new int(3));
	MyWeakPtr<int> w(a);
	assert(!w.expired());
	assert(w.weak_count() == 1);
	auto w2 = w;
	assert(!w2.expired());
	assert(w2.weak_count() == 2);
	assert(w.weak_count() == 2);
}
}

}

int main() {
	using namespace mysharedptr;
	test_empty();
	test_lock();
	test_reset();
	test_counter_values();
	test_empty_copy();
	test_copy();
}
