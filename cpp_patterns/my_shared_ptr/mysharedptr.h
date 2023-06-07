#pragma once 

#include <utility>
#include <atomic>

template <typename T>
struct MyWeakPtr;


template <typename T>
class MySharedPtr {

	struct ControlBlock {
		std::atomic<unsigned> counter = 0; // counter of shared pointers
		std::atomic<unsigned> weak = 0;    // counter of weak pointers
		T* ptr = nullptr;
	};
public:
	MySharedPtr() = default;

	MySharedPtr(T* p) {
		if (!p)
			return;
		block = new ControlBlock(); // TODO how to allocate two at once?

		block->ptr = p;
		block->counter = 1;
	}

	MySharedPtr(const MySharedPtr& other) {
		block = other.block;
		increment();
	}

	MySharedPtr(MySharedPtr&& other) {
		*this = std::move(other);
	}

	MySharedPtr& operator=(MySharedPtr&& other) {
		std::swap(block, other.block);
		return *this;
	}

	MySharedPtr& operator=(const MySharedPtr& other) {
		if (other.block == block) {
			return *this;
		}
		MySharedPtr copy = std::move(*this);

		block = other.block;
		increment();
		return *this;
	}

	~MySharedPtr() {
		decrement();
	}

	bool operator==(const MySharedPtr& other) const {
		return block == other.block;
	}

	T& operator*() {
		return *block->ptr;
	}

	T* operator->() {
		return block->ptr;
	}

	int use_count() const {
		if (!block) 
			return 0;
		
		return block->counter;
	}

	void reset(T* arg) {
		*this = MySharedPtr(arg);
	}

	operator bool() const {
		if (block && block->counter > 0)
			return block->ptr;
		return false;
	}

	T* get() {
		if (!block) 
			return nullptr;
		return block->ptr;
	}

	private:
	void increment() {
		if (!block) 
			return;
		block->counter++;
	}

	void decrement() {
		if (!block) 
			return;
		block->counter--;

		if (block->counter == 0) {
			if (block->weak == 0) {
				delete block->ptr;
				delete block;
			}
		}
	}

	// MEMBERS
	ControlBlock* block=nullptr;

	friend class MyWeakPtr<T>;
};
