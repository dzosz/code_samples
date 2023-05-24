#pragma once 

#include <utility>

template <typename T>
struct MyWeakPtr;

template <typename T>
class MySharedPtr {
public:
	MySharedPtr() = default;

	MySharedPtr(T* p) {
		if (p == nullptr)
			return;

		ptr = p;
		counter = new int[2](); // use single allocation instead of two
		*counter = 1;
		weak = &counter[1];
	}

	MySharedPtr(const MySharedPtr& other) {
		//if (!other.ptr) 
		//	return;

		ptr = other.ptr;
		counter = other.counter;
		weak    = other.weak;
		increment();
	}

	MySharedPtr(MySharedPtr&& other) {
		*this = std::move(other);
	}

	MySharedPtr& operator=(MySharedPtr&& other) {
		std::swap(counter, other.counter);
		std::swap(ptr, other.ptr);
		std::swap(weak, other.weak);
		return *this;
	}

	MySharedPtr& operator=(const MySharedPtr& other) {
		if (other.ptr == ptr) {
			return *this;
		}
		MySharedPtr copy = std::move(*this);

		counter = other.counter;
		ptr = other.ptr;
		increment();
		return *this;
	}

	~MySharedPtr() {
		if (!counter) {
			return;
		}
		decrement();
	}

	bool operator==(const MySharedPtr& other) const {
		return other.ptr == ptr;
	}

	T& operator*() {
		return *ptr;
	}

	T* operator->() {
		return ptr;
	}

	int use_count() const {
		if (!counter) 
			return 0;
		
		return *counter;
	}

	void reset(T* arg) {
		*this = MySharedPtr(arg);
	}

	operator bool() const {
		if (counter && (*counter) > 0)
			return ptr;
		return false;
	}

	T* get() {
		return ptr;
	}

	private:
	void increment() {
		if (!counter) 
			return;
		(*counter)++;
	}

	void decrement() {
		if (!counter) 
			return;
		(*counter)--;

		if ((*counter) == 0) {
			if (!weak || weak == 0) {
				delete[] counter;
				delete ptr ;
			}
		}
	}

	// MEMBERS
	mutable int* counter = 0; // counter of shared pointers
	mutable int* weak = 0;    // counter of weak pointers
	T* ptr = 0;

	friend class MyWeakPtr<T>;
};
