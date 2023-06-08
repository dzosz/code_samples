#pragma once 

#include <atomic>
#include <variant>
#include <memory>
#include <utility>

namespace mysharedptr
{

template <typename T>
struct MyWeakPtr;


template <typename T>
class MySharedPtr {

	struct ControlBlock {
		explicit ControlBlock(T* data) : ptr{std::unique_ptr<T>(data)} { }

		template <typename ...Ts>
		explicit ControlBlock(Ts&& ...args) : ptr{T(args...)} {}

		T* get_ptr() {
			if (auto val = std::get_if<T>(&ptr)) {
				return val;
			}
			return std::get_if<std::unique_ptr<T>>(&ptr)->get();
		}

		T& get_ref() {
			return *this->get_ptr();
		}

		void set(T* instance) {
			std::unique_ptr<T> tmp(instance);
			this->ptr = std::move(tmp);
		}

		unsigned count() const {
			return counter;
		}

		unsigned weak_count() const {
			return weak;
		}

		void increment() {
			counter++;
		}

		void increment_weak() {
			weak++;
		}

		bool decrement() {
			counter--;

			if (counter == 0) {
				this->set(nullptr); // free
				if (weak == 0) {
					delete this;
					return true;
				}
			}
			return false;
		}

		bool decrement_weak() {
			weak--;
			if (weak == 0 && counter == 0) {
				delete this;
				return true;
			}
			return false;
		}
		private:
		std::atomic<unsigned> counter = 1; // counter of shared pointers
		std::atomic<unsigned> weak = 0;    // counter of weak pointers
		std::variant<std::unique_ptr<T>, T> ptr;
		// TODO add deleter function

	};
public:
	explicit MySharedPtr() = default;

	template <typename ...Ts>
	explicit MySharedPtr(Ts&&... args) {
		block = new ControlBlock(args...);
	}

	explicit MySharedPtr(T* p) {
		if (!p)
			return;
		block = new ControlBlock(p);
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
		return block->get_ref();
	}

	T* operator->() {
		return block->get_ptr();
	}

	unsigned use_count() const {
		if (!block) 
			return 0;
		
		return block->count();
	}

	void reset(T* arg) {
		*this = MySharedPtr(arg);
	}

	operator bool() const {
		if (block && block->count() > 0)
			return block->get_ptr();
		return false;
	}

	T* get() {
		if (!block) 
			return nullptr;
		return block->get_ptr();
	}

	private:
	void increment() {
		if (!block) 
			return;
		block->increment();
	}

	void decrement() {
		if (!block) 
			return;

		block->decrement();
		block = nullptr;
	}

	// MEMBERS
	ControlBlock* block=nullptr;

	friend class MyWeakPtr<T>;
};

template <typename T, typename ...Ts>
MySharedPtr<T> make_shared(Ts&& ...args) {
	return MySharedPtr<T>(args...);
}

}
