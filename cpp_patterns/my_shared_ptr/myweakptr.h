#pragma once 

#include "mysharedptr.h"
#include <utility>

template <typename T>
struct MyWeakPtr {
	MyWeakPtr() = default;

	MyWeakPtr(const MySharedPtr<T>& shd) {
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

	MyWeakPtr& operator=(const MyWeakPtr& other) {
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
		return !block || block->counter == 0;
	}

	//return use count of shared ptr
	int use_count() const {
		if (block)
			return block->counter;
		return 0;
	}

	int weak_count() const {
		if (block)
			return block->weak;
		return 0;
	}

	private:
	void increment() {
		if (block) 
			block->weak++;
	}

	void decrement() {
		if (!block)
			return;

		block->weak--;
		if (block->weak == 0 && block->counter == 0) {
			delete block->ptr;
			delete block;

			block = nullptr;
		}
	}

	typename MySharedPtr<T>::ControlBlock* block = nullptr;
};
