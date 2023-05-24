#pragma once 

#include "mysharedptr.h"
#include <utility>

template <typename T>
struct MyWeakPtr {
	MyWeakPtr() = default;

	MyWeakPtr(const MySharedPtr<T>& shd) {
		counter = shd.counter;
		weak = shd.weak;
		ptr = shd.ptr;

		increment();
	}

	MyWeakPtr(MyWeakPtr<T>& other) {
		counter = other.counter;
		weak = other.weak;
		ptr = other.ptr;
		increment();
	}


	MyWeakPtr(MyWeakPtr<T>&& other) {
		*this = std::move(other);
	}

	MyWeakPtr& operator=(const MyWeakPtr& other) {
		counter = other.counter;
		weak = other.weak;
		ptr = other.ptr;
		increment();
		return *this;
	}

	MyWeakPtr& operator=(MyWeakPtr&& other) {
		std::swap(counter, other.counter);
		std::swap(ptr, other.ptr);
		std::swap(weak, other.weak);
		return *this;
	}


	~MyWeakPtr() {
		decrement();
	}

	MySharedPtr<T> lock() {
		MySharedPtr<T> tmp;
		tmp.counter = counter;
		tmp.weak = weak;
		tmp.ptr = ptr;
		tmp.increment();

		return tmp;
	}

	bool expired() const {
		return !counter || (*counter) == 0;
	}

	//return use count of shared ptr
	int use_count() const {
		if (counter)
			return *counter;
		return 0;
	}

	int weak_count() const {
		if (weak)
			return *weak;
		return 0;
	}

	private:
	void increment() {
		if (weak) 
			(*weak)++;
	}

	void decrement() {
		if (!weak)
			return;

		(*weak)--;
		if (*weak == 0 && *counter == 0) {
			delete[] counter;
			// delete weak; // allocated together with counter
			delete ptr;

			weak = 0;
			counter = 0;
			ptr = 0;
		}
	}

	mutable int* counter = 0; // counter of shared pointers
	mutable int* weak = 0;    // counter of weak pointers
	T* ptr = 0;
};
