.PHONY: all test

all: seqlock test

seqlock: seqlock.rs
	rustc seqlock.rs

seqlock_test: seqlock.rs
	rustc -C debuginfo=2 -C opt-level=3 --test seqlock.rs -o seqlock_test

test: seqlock_test
	./seqlock --show-output
