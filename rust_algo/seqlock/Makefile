.PHONY: all test

all: libseqlock.rlib test

libseqlock.rlib: seqlock.rs
	rustc --crate-type=lib seqlock.rs

seqlock_test: seqlock.rs
	rustc -C debuginfo=2 -C opt-level=3 --test seqlock.rs -o seqlock_test

test: seqlock_test
	./seqlock_test --show-output
