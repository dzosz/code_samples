pub mod seqlock {

    use std::sync::atomic::{AtomicUsize, Ordering, fence};

    use std::mem::MaybeUninit;

    use std::cell::Cell;

    // use std::ptr::NonNull;
    //use std::ops::{Deref, DerefMut};

    type Counter = AtomicUsize;

    #[repr(align(64))]
    pub struct SeqLock<T> {
        iteration: Counter,
        item: Cell<T>, // modified
    }

    // required to make Cell safe for multithreaded access
    unsafe impl<T> Send for SeqLock<T> {}
    unsafe impl<T> Sync for SeqLock<T> {}

    impl<T: Copy> SeqLock<T> {
        pub fn new(val: T) -> SeqLock<T> {
            SeqLock {
                item: val.into(),
                iteration: AtomicUsize::new(0),
            }
        }

        pub fn get_writer(&self) -> SeqLockWriter<T> {
            let obj = SeqLockWriter {
                item: &self.item,
                iteration: &self.iteration,
            };
            obj._start_write();
            obj
        }
        pub fn get_reader(&self) -> SeqLockReader<T> {
            SeqLockReader {
                item: &self.item,
                iteration: &self.iteration,
            }
        }
    }

    pub struct SeqLockWriter<'a, T: Copy> {
        iteration: &'a Counter,
        item: &'a Cell<T>,
    }

    impl<T: Copy> SeqLockWriter<'_, T> {
        // consuming. single threaded
        pub fn write(self, val: T) {
            self.item.set(val);
            //std::ptr::write(self.item.as_ptr(), val); // TODO some pople use 'std::ptr::write_volatile' here
        }

        // consuming.
        pub fn write_with(self, closure: impl Fn(*mut T)) {
            closure(self.item.as_ptr());
        }

        /* A release operation only needs to prevent preceding memory operations from being reordered past itself,
         * but a release fence must prevent preceding memory operations from being reordered past all subsequent writes
         */
        fn _start_write(&self) {
            debug_assert!(
                self.iteration.load(Ordering::Relaxed) % 2 == 0,
                "single writer allowed"
            );

            self.iteration.fetch_add(1, Ordering::Relaxed); // acquire not required because writer is single threaded.
                                                            // also acquire is available for read only
            fence(Ordering::Release);
        }

        fn _end_write(&self) {
            debug_assert!(
                self.iteration.load(Ordering::Relaxed) % 2 == 1,
                "single writer allowed"
            );

            self.iteration.fetch_add(1, Ordering::Release);
        }
    }

    impl<T: Copy> Drop for SeqLockWriter<'_, T> {
        fn drop(&mut self) {
            self._end_write();
        }
    }

    pub struct SeqLockReader<'a, T> {
        iteration: &'a Counter,
        item: &'a Cell<T>,
    }

    impl<T: Copy> SeqLockReader<'_, T> {
        pub fn read(&self) -> T {
            unsafe {
                let mut val: MaybeUninit<T> = MaybeUninit::uninit();
                while !self._try_read(val.as_mut_ptr()) {
                    // std::thread::yield_now();
                }
                *val.as_mut_ptr()
            }
        }

        pub fn read_into(&self, val : &mut T) {
			while !self._try_read(val as *mut _) {
				//std::thread::yield_now();
			}
        }

        /*
        pub fn try_read(&self) -> Option<T> {
            let val = Default::default();
            unsafe {
                let my_ref = val.get_or_insert(&mut *buff.as_mut_ptr()); // workaround to get ref to optional cheaply
                let success = self._try_read(*my_ref);
                if !success {
                    *val = None;
                }
            }
        }
        */

        pub fn try_read_into(&self, val: &mut Option<&mut T>) {
            let mut buff: MaybeUninit<T> = MaybeUninit::uninit();
            unsafe {
                let my_ref = val.get_or_insert(&mut *buff.as_mut_ptr()); // workaround to get ref to optional cheaply
                let success = self._try_read(*my_ref);
                if !success {
                    *val = None;
                }
            }
        }

        fn _try_read(&self, val: *mut T) -> bool {
            let prev = self.iteration.load(Ordering::Acquire);
            if prev % 2 == 0 {
                unsafe {
                    *val = self.item.get();
                }
                //*val = *self.item.as_ptr(); // TODO might want to use 'std::ptr::read_volatile' here...
                fence(Ordering::Acquire);
                return prev == self.iteration.load(Ordering::Relaxed);
            }
            return false;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::convert::TryInto;
        use std::sync::Arc;
        use std::thread;

        struct TestWriter {
            data: Vec<u64>,
        }

        impl TestWriter {
            fn new(num: usize) -> TestWriter {
                let mut obj = TestWriter { data: vec![0; num] };
                obj.generate_consecutive_numbers(0);
                obj
            }

            fn generate_consecutive_numbers(&mut self, start: u64) {
                let mut idx = 0;
                self.data.iter_mut().for_each(|e| {
                    *e = idx + start;
                    idx += 1;
                });
            }

            fn are_numbers_in_increasing_order(data: &[u64]) {
                data.iter().enumerate().skip(1).fold(data[0], |prev, (i, next)| {
                    if prev + 1 != *next {
                        panic!("idx={} not equal {:?} != {:?}", i, prev, *next);
                    }
                    *next
                });
            }
        }

        #[test]
        fn test_single_consumer_one_cacheline() {
            const ARRAY_SIZE: usize = 8;

            let mut data_writer = TestWriter::new(ARRAY_SIZE);
            let my_lock = Arc::new(SeqLock::<[u64; ARRAY_SIZE]>::new(
                data_writer.data.clone().try_into().unwrap(),
            ));

            let iterations = 100000000;

            let lock_writer = my_lock.clone();
            let writer_thread = thread::spawn(move || {
                for i in 0..iterations {
                    data_writer.generate_consecutive_numbers(i);
                    lock_writer.get_writer().write_with(|item| unsafe {
                        *item = data_writer.data.as_slice().try_into().unwrap();
                    });
                }
            });

            {
                let reader = my_lock.get_reader();
                for _ in 0..iterations {
                    let value = reader.read();
                    TestWriter::are_numbers_in_increasing_order(&value);
                }
            }

            writer_thread.join().unwrap();
        }

        #[test]
        fn test_single_consumer_one_cacheline_try() {
            const ARRAY_SIZE: usize = 8;

            let mut data_writer = TestWriter::new(ARRAY_SIZE);
            let my_lock = Arc::new(SeqLock::<[u64; ARRAY_SIZE]>::new(
                data_writer.data.clone().try_into().unwrap(),
            ));

            let iterations = 100000000;

            let lock_writer = my_lock.clone();
            let writer_thread = thread::spawn(move || {
                for i in 0..iterations {
                    data_writer.generate_consecutive_numbers(i);
                    lock_writer.get_writer().write_with(|item| unsafe {
                        *item = data_writer.data.as_slice().try_into().unwrap();
                    });
                }
            });

            {
                let mut my_optional = Default::default();
                let reader = my_lock.get_reader();
                for _ in 0..iterations {
                    reader.try_read_into(&mut my_optional);
                    match my_optional {
                        Some(ref value) => {TestWriter::are_numbers_in_increasing_order(*value); },
                        _ => {},
                    }

                }
            }

            writer_thread.join().unwrap();
        }
    }
} // mod seqlock
