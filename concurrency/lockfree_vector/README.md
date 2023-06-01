# Lockfree Vector "Dynamically Resizable Arrays"
## Damian Dechev, Peter Pirkelbauer, and Bjarne Stroustrup

Implementation of lockfree vector in Rust based on original paper from 2006

## Concurrent data structures
Designing data structures is an art of compromises. Standard libraries ship most universal algorithms and structures that perform well in average use cases. The essence C++ standard library is "Zero cost abstractions" or "Don't pay for what you use" (however some structures like std::unordered_map with std::list buckets break this principle).  Concurrent data structures like presented vector are expected to outperform them in specific multithreaded scenarios however with a cost in singlethreaded environment that makes them unsuitable for average use. The tradeoffs affect all dimensions of computer programs: memory usage, read/write latency and throughput. 

## Glossary
Mutex - mutual exclusive synchronization
Lockfree - concurent access that guarantees progress with unknown number of steps/instructions
Waitfree - concurent access that guarantess progress within specified number of steps/instructions

## Implementation overview
Lockfree vector achieves thread safety by splitting moving parts and accessing them with atomic operation Compare And Swap (CAS). In order to support push_back() and pop_back() a heap allocated (protected by reference counter or other strategy) struct Descriptor is used. Descriptor plays the role of queueing a change event that later modifies underlying buffer. Write() and read() operations do not need to go through Descriptor.

In original implementation descriptor consists of reference counter (for lifeteime management), new store value, previous value and index of change. Previous value is required for conditional CAS execution, however this approach is susceptible to 'ABA' errors. CAS operation suceeds if previous pointer is the same, but there's no guarantee that this value was not changed in the meantime.

The paper does go into the details of the strategy for Descriptor object lifetime management, so two object reclamation were chosen for this implementation
* spinlock protected descriptor (uses counter that is inside Descriptor struct)
* Epoch based reclamation (Rust crossbeam library was used)

### Data layout
Classic vector is made up of 3 parts: size, capacity and pointer to contiguous data. In Lockfree Vector allocated data is not a single buffer, but a two level array - array of pointers to increasingly sized buffers. The initial memory bucket has been arbitrarily chosen to 8 elements and use growth factor of 2 which requires additional log2N memory.

### Use cases
* Low throughput scenarios that modify few items at once or in scattered manner
* High variance in access patterns with modifications
* Growth without known upper bound size
* LIFO ordered changes

## Data requirements
Stored objects need to:
* Fit in atomic instruction
* be "Plain Old Data" or in Rust terms implement Copy trait. This implementation generalizes uses `usize` type for its elements for simplicity.

### Supported vector operations
* push_back(elem)
* pop_back()
* reserve(size) / resize(size)
* size()
* read(index)
* write(index, elem)

### Unsupported vector operations
* erase(index)
* any copy, insert or delete ranged operations
* swap() (possible to implement)
* clear() (can be performed in multiple steps with pop_back())

## Compile instructions
* Build
  * `rustup default nightly` 
  * `cargo build`
* Run tests
  * `cargo test`
* Run benchmarks
  * `cargo +nightly bench`

## Benchmarks
Synthethic benchmarks of lockfree vector show a staggering advantage over using vector with mutexes.
* TODO
