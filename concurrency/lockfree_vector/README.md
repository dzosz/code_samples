# Lockfree Vector "Dynamically Resizable Arrays"
## Damian Dechev, Peter Pirkelbauer, and Bjarne Stroustrup

Implementation of lockfree vector in Rust based on original paper from 2006

## Concurrent data structures
Designing data structures is an art of compromises. Standard libraries ship most universal algorithms and structures that perform well in all circumstances. They theme for C++ standard library are "Zero cost abstractions" and "Don't pay for what you use" (however some structures like std::unordered_map with std::list buckets break this principle). Concurrent data structures like presented vector are expected to perform well in multithreaded environments but worse in singlethreaded use cases. The tradeoffs affect memory usage, read/write latency or throughput.

## Glossary
Mutex - mutual exclusive synchronization
Lockfree - concurent access that guarantees progress with unknown number of steps/instructions
Waitfree - concurent access that guarantess progress within specified number of steps/instructions

## Implementation overview

### Use cases
* Low throughput scenarios that modify few items at once
* High concurrency without known upper bound size

## Data requirements
Stored objects need to:
* Fit in atomic instruction
* be "Plain Old Data" or in Rust terms implement Copy trait. This implementation uses `usize` type for its elements for simplicity.

### Data layout

### Supported vector operations
* push_back(elem)
* pop_back()
* reserve()/resize()
* size()
* read(index)
* write(index, elem)

### Unsupported vector operations
* Erase(index)

## QA
* Why memory array is defined as array of raw pointer instead of array of atomics?
  Initializing arrays requires Copy trait which is not implemented for AtomicPtr. This doesn't matter as long as pointers and data is accessed in atomic way (using AtomicPtr structure in this implementation).
* 
