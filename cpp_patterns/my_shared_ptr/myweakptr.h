#pragma once 

#include "mysharedptr.h"
#include <utility>

namespace mysharedptr
{

template <typename T>
class MyWeakPtr {
	public:
	MyWeakPtr() = default;

	explicit MyWeakPtr(const MySharedPtr<T>& shd) {
		block = shd.block;
		increment();
	}

	MyWeakPtr(MyWeakPtr<T>& other) {
		block = other.block;
		increment();
	}

	MyWeakPtr(MyWeakPtr<T>&& other) {
		*this = std::move(other);
	}

	MyWeakPtr<T>& operator=(const MyWeakPtr<T>& other) {
		auto old = std::move(*this);
		block = other.block;
		increment();
		return *this;
	}

	MyWeakPtr& operator=(MyWeakPtr&& other) {
		std::swap(block, other.block);
		return *this;
	}

	~MyWeakPtr() {
		decrement();
	}

	MySharedPtr<T> lock() {
		MySharedPtr<T> tmp;
		tmp.block = block;
		tmp.increment();

		return tmp;
	}

	bool expired() const {
		return !block || block->count() == 0;
	}

	//return use count of shared ptr
	unsigned use_count() const {
		if (block)
			return block->count();
		return 0;
	}

	unsigned weak_count() const {
		if (block)
			return block->weak_count();
		return 0;
	}

	private:
	void increment() {
		if (block) 
			block->increment_weak();
	}

	void decrement() {
		if (!block)
			return;

		block->decrement_weak();
		block = nullptr;
	}

	typename MySharedPtr<T>::ControlBlock* block = nullptr;
};

}
