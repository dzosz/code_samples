#![feature(vec_into_raw_parts)]
#![feature(associated_type_defaults)]
#![feature(atomic_from_ptr, atomic_from_mut, pointer_is_aligned)]
#![feature(test)]
#![feature(get_mut_unchecked)]
#![feature(local_key_cell_methods)]

#[macro_use]
mod log;
mod bench;
mod descriptor;
mod strategy;

pub mod lockfree_vec {
    use crate::descriptor::Descriptor;
    use crate::descriptor::WriteDescriptor;
    use crate::strategy::SpinlockDescriptorStrategy;
    use crate::strategy::EpochGarbageCollectionStrategy;
    use crate::strategy::Strategy;
    use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

    type DataT = usize; // TODO should be generic over vector

    const FIRST_BUCKET_SIZE: usize = 8; // must be mutliple of 2
                                        //
    fn highest_bit_index(num: usize) -> usize {
        //((1 << (usize::BITS - num.leading_zeros()) >> 1) as usize).trailing_zeros() as usize
        ((usize::BITS - num.leading_zeros()) - 1) as usize // 0 unsupported
    }

    fn bucket_size(bucket: usize) -> usize {
        1 << (highest_bit_index(FIRST_BUCKET_SIZE) + bucket)
    }

    /* TODO
       Initializing arrays requires Copy trait which is not implemented for AtomicPtr. This doesn't matter as long as pointers and data is accessed in atomic way (using AtomicPtr structure in this implementation).
    */
    /* TODO
     * make LockfreeVec generic over DataT
     */

    #[repr(align(64))]
    pub struct LockfreeVec {
        //descriptor: AtomicPtr<Descriptor>, // moved to strategy
        memory: Vec<AtomicPtr<AtomicUsize>>, // can be static array too
        //strategy: SpinlockDescriptorStrategy,
        strategy: Box<dyn Strategy>,
    }

    // make safe for multithreaded access
    unsafe impl Send for LockfreeVec {}
    unsafe impl Sync for LockfreeVec {}

    impl LockfreeVec {
        pub fn new() -> LockfreeVec {
            assert_eq!(FIRST_BUCKET_SIZE % 2, 0);

            LockfreeVec {
                memory: (0..64).map(|_| AtomicPtr::new(std::ptr::null_mut())).collect(),
                strategy: Box::new(EpochGarbageCollectionStrategy::new()),
                //strategy: Box::new(SpinlockDescriptorStrategy::new()),
            }
        }

        pub fn push_back(&self, elem: DataT) {
            let new_desc = self.strategy.alloc();
            let new_desc_ref = unsafe { new_desc.as_mut().unwrap() };

            let guard = self.strategy.guard();
            loop {
                let desc = self.strategy.access(&guard);
                let desc_ref = unsafe { desc.as_ref().unwrap() };
                self.complete_write(desc_ref);
                {
                    let (bucket, _) = Self::get_bucket_and_pos_at(desc_ref.size);
                    let bucket_ptr = self.get_bucket(bucket).load(Ordering::SeqCst);
                    if bucket_ptr.is_null() {
                        self.alloc_bucket(bucket);
                    }
                }

                new_desc_ref.pending = Some(WriteDescriptor::new(
                        self.read(desc_ref.size), elem, desc_ref.size));
                new_desc_ref.size = desc_ref.size + 1;

                if self.strategy.swap(desc, new_desc, &guard) {
                    self.complete_write(new_desc_ref);
                    self.strategy.dealloc(desc, &guard);
                    self.strategy.release_access(new_desc);
                    return;
                } else {
                    self.strategy.release_access(desc);
                }
            }
        }

        pub fn pop_back(&self) -> Option<DataT> {
            let mut new_desc : *mut Descriptor = std::ptr::null_mut();

            let guard = self.strategy.guard();
            loop {
                let desc = self.strategy.access(&guard);
                let desc_ref = unsafe { desc.as_ref().unwrap() };

                if desc_ref.size == 0 {
                    self.strategy.release_access(desc);
                    return None;
                }
                if new_desc.is_null() {
                    new_desc = self.strategy.alloc();
                }
                let new_desc_ref = unsafe { new_desc.as_mut().unwrap() };

                self.complete_write(desc_ref);

                let elem = self.read(desc_ref.size - 1);
                new_desc_ref.size = desc_ref.size - 1;
                new_desc_ref.pending = None;

                if self.strategy.swap(desc, new_desc, &guard) {
                    self.complete_write(new_desc_ref);
                    self.strategy.release_access(new_desc);
                    self.strategy.dealloc(desc, &guard);
                    return Some(elem);
                } else {
                    self.strategy.release_access(desc);
                }
                // continue
            }
        }

        pub fn read(&self, i: usize) -> DataT {
            //debug_assert!(i <= self.size()); // can't guarantee anything here
            self.at(i).load(Ordering::SeqCst)
        }

        pub fn write(&self, i: usize, elem: DataT) {
            debug_assert!(i <= self.size()); // push_back writes after last element
            self.at(i).store(elem, Ordering::SeqCst);
        }

        pub fn reserve(&self, size: usize) {
            let guard = self.strategy.guard();
            let cur_size = self.strategy.descriptor(&guard).size;
            let (mut i, _) = Self::get_bucket_and_pos_at(cur_size - (cur_size > 0) as usize);
            if cur_size > 0 {
                i = i + 1; // we want to allocate only next bucket
            }
            let (bucket, _) = Self::get_bucket_and_pos_at(size - 1);
            while i <= bucket {
                self.alloc_bucket(i);
                i = i + 1;
            }
        }

        pub fn size(&self) -> usize {
            let guard = self.strategy.guard();
            let desc = self.strategy.descriptor(&guard);
            match desc.pending {
                Some(ref writeop) => desc.size - (!writeop.completed.load(Ordering::Relaxed) as usize),
                _ => desc.size,
            }
        }

        fn at(&self, i: usize) -> &AtomicUsize {
            let (bucket, idx) = Self::get_bucket_and_pos_at(i);
            let bucket_ptr = self.get_bucket(bucket).load(Ordering::SeqCst);
            let item_ptr = unsafe { bucket_ptr.offset(idx as isize) };
            let item = unsafe { AtomicUsize::from_ptr(item_ptr as *mut usize) };
            item
        }

        fn complete_write(&self, desc: &Descriptor) {
            if let Some(ref writeop) = desc.pending.as_ref() {
                if !writeop.completed.load(Ordering::Relaxed) {
                    let ref value = self.at(writeop.pos);
                    match value.compare_exchange(
                        writeop.old_value,
                        writeop.new_value,
                        Ordering::SeqCst,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => { }
                        _ => {
                            // different thread succeeded?
                        }
                    }
                    writeop.completed.store(true, Ordering::Relaxed);
                }
            }
        }

        fn alloc_bucket(&self, bucket: usize) {
            let bucket_ptr = self.get_bucket(bucket);
            if !bucket_ptr.load(Ordering::Relaxed).is_null() {
                return;
            }

            let bucket_size = bucket_size(bucket);
            let mem: Vec<AtomicUsize> = Vec::with_capacity(bucket_size);
            let (mem_ptr, _, _) = mem.into_raw_parts();
            let null = std::ptr::null_mut();
            match bucket_ptr.compare_exchange(null, mem_ptr, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(_) => {}
                Err(_) => unsafe {
                    // different thread succeeded
                    drop(Vec::from_raw_parts(mem_ptr, 0, bucket_size));
                },
            }
        }

        fn get_bucket(&self, bucket: usize) -> &AtomicPtr<AtomicUsize> {
            unsafe { self.memory.get_unchecked(bucket) }
        }

        fn get_bucket_and_pos_at(i: usize) -> (usize, usize) {
            let pos = i + FIRST_BUCKET_SIZE;
            let hibit = highest_bit_index(pos);
            let idx = pos ^ (1 << hibit);
            let bucket = hibit - highest_bit_index(FIRST_BUCKET_SIZE);
            return (bucket, idx);
        }
    }

    impl Drop for LockfreeVec {
        fn drop(&mut self) {
            unsafe {
                //drop(Box::from_raw(self.descriptor.load(Ordering::SeqCst)));
                for bucket in 0..self.memory.len() {
                    let bucket_ptr = self.get_bucket(bucket).load(Ordering::SeqCst);
                    if bucket_ptr.is_null() {
                        break; // next buckets are guaranteed to be free
                    }
                    let bucket_size = bucket_size(bucket);
                    drop(Vec::from_raw_parts(bucket_ptr, bucket_size, bucket_size));
                }
            }
        }
    }


    #[cfg(test)]
    mod tests {
        use super::*;
        use std::sync::Arc;
        use std::thread;

        #[test]
        fn test_indexing() {
            assert_eq!(
                (0, FIRST_BUCKET_SIZE - 1),
                LockfreeVec::get_bucket_and_pos_at(FIRST_BUCKET_SIZE - 1)
            );
            assert_eq!((0, 0), LockfreeVec::get_bucket_and_pos_at(0));
            assert_eq!((0, 1), LockfreeVec::get_bucket_and_pos_at(1));
            assert_eq!((0, 2), LockfreeVec::get_bucket_and_pos_at(2));
            assert_eq!((0, 3), LockfreeVec::get_bucket_and_pos_at(3));
            assert_eq!((0, 7), LockfreeVec::get_bucket_and_pos_at(7));
            assert_eq!(
                (1, 0),
                LockfreeVec::get_bucket_and_pos_at(FIRST_BUCKET_SIZE)
            );
            assert_eq!(
                (1, FIRST_BUCKET_SIZE),
                LockfreeVec::get_bucket_and_pos_at(FIRST_BUCKET_SIZE * 2)
            );
            assert_eq!(
                (2, 0),
                LockfreeVec::get_bucket_and_pos_at(FIRST_BUCKET_SIZE * 3)
            );
        }

        #[test]
        fn test_highest_bit_index() {
            // assert_eq!(64, highest_bit_index(0b0)); // TODO 0 is unsupported ?
            assert_eq!(0, highest_bit_index(0b1));
            assert_eq!(1, highest_bit_index(0b11));
            assert_eq!(2, highest_bit_index(0b111));
            assert_eq!(2, highest_bit_index(0b101));
            assert_eq!(2, highest_bit_index(0b100));
            assert_eq!(7, highest_bit_index(0b10000001));
        }

        #[test]
        fn test_bucket_size() {
            assert_eq!(bucket_size(0), FIRST_BUCKET_SIZE);
            assert_eq!(bucket_size(1), FIRST_BUCKET_SIZE * 2);
            assert_eq!(bucket_size(2), FIRST_BUCKET_SIZE * 4);
        }

        #[test]
        fn test_push_back() {
            let vec = LockfreeVec::new();
            for i in 0..512 {
                vec.push_back(i);
                assert_eq!(vec.read(i), i);
            }
        }

        #[test]
        fn test_pop_back() {
            let vec = LockfreeVec::new();
            let iterations = 20000;
            for i in 0..iterations {
                vec.push_back(i);
            }
            for i in iterations..0 {
                let item = vec.pop_back();
                assert_eq!(i, item.unwrap());
            }
        }

        #[test]
        fn test_size() {
            let vec = LockfreeVec::new();
            assert_eq!(vec.size(), 0);
            vec.push_back(1);
            assert_eq!(vec.size(), 1);
            vec.push_back(2);
            assert_eq!(vec.size(), 2);
        }

        #[test]
        fn test_read() {
            let vec = LockfreeVec::new();
            let iterations = 512;
            for i in 0..iterations {
                vec.push_back(i);
            }
            for i in 0..iterations {
                assert_eq!(vec.read(i), i);
            }
        }

        #[test]
        fn test_write() {
            let vec = LockfreeVec::new();
            let iterations = 512;
            for i in 0..iterations {
                vec.push_back(0);
            }
            for i in 0..iterations {
                vec.write(i, i);
            }
            for i in 0..iterations {
                assert_eq!(vec.read(i), i);
            }
        }

        #[test]
        fn test_reserve_allocates_correct_bucket() {
            // TODO parametrize this test
            let vec = LockfreeVec::new();

            let check_bucket = |new_size| {
                vec.reserve(new_size);
                let (bucket, _) = LockfreeVec::get_bucket_and_pos_at(new_size - 1);
                assert_eq!(
                    vec.get_bucket(bucket).load(Ordering::Relaxed).is_null(),
                    false
                );
                assert_eq!(
                    vec.get_bucket(bucket + 1).load(Ordering::Relaxed).is_null(),
                    true
                );
            };

            check_bucket(7);
            check_bucket(8);
            check_bucket(9);
            check_bucket(16);
            check_bucket(32);
            check_bucket(47);
            check_bucket(48);
            check_bucket(49);
        }

        #[test]
        fn test_concurrent_push_pop() {
            let vec = Arc::new(LockfreeVec::new());
            let vec2 = vec.clone();
            let iterations = 200000;

            let p1 = thread::spawn(move || {
                for i in 0..iterations {
                    vec2.push_back(i);
                }
            });

            {
                // concurrently pop back
                let mut verify_vec = Vec::new();
                for _ in 0..iterations {
                    loop {
                        let item = vec.pop_back();

                        //assert_eq!(item.is_some(), true);
                        if item.is_some() {
                            verify_vec.push(item.unwrap());
                            break;
                        }
                    }
                }
                verify_vec.sort();
                assert_eq!(verify_vec.len(), iterations);
                for i in 0..iterations {
                    assert_eq!(verify_vec[i], i);
                }
            };
            p1.join().unwrap();
            assert_eq!(vec.size(), 0);
        }

        #[test]
        fn test_concurrent_push() {
            let vec = Arc::new(LockfreeVec::new());
            let vec2 = vec.clone();
            let vec3 = vec.clone();
            let iterations = 20000;
            let w1 = thread::spawn(move || {
                for i in 0..iterations {
                    vec2.push_back(i);
                }
            });
            let w2 = thread::spawn(move || {
                for i in 0..iterations {
                    vec3.push_back(i);
                }
            });
            for i in 0..iterations {
                vec.push_back(i);
            }
            w1.join().unwrap();
            w2.join().unwrap();
            assert_eq!(vec.size(), iterations * 3);
        }
    }
} // mod
