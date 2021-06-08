pub mod seqlock {

use std::sync::atomic::{AtomicUsize, Ordering};

use std::mem::MaybeUninit;

// use std::ptr::NonNull;

//use std::ops::{Deref, DerefMut};

type Counter = AtomicUsize;

use std::cell::Cell;

#[repr(align(64))]
pub struct SeqLock<T> {
    iteration: Counter,
    item: Cell<T>, // modified
}

// required to make Cell safe for multithreaded access
unsafe impl<T> Send for SeqLock<T> {}
unsafe impl<T> Sync for SeqLock<T> {}

impl<T> SeqLock<T> {
    pub fn new(val: T) -> SeqLock<T> {
        SeqLock {
            item: val.into(),
            iteration: AtomicUsize::new(0),
        }
    }

    pub fn get_writer(&self) -> SeqLockWriter<T> {
        SeqLockWriter {
            item: &self.item,
            iteration: &self.iteration,
        }
    }
    pub fn get_reader(&self) -> SeqLockReader<T> {
        SeqLockReader {
            item: &self.item,
            iteration: &self.iteration,
        }
    }
}

pub struct SeqLockWriter<'a, T> {
    iteration: &'a Counter,
    item: &'a Cell<T>,
}

impl<'a, T: Copy> SeqLockWriter<'a, T> {
    // single writer  only
    pub fn write(&mut self, val: T) {
        self._start_write();
        unsafe {
            std::ptr::write(self.item.as_ptr(), val); // TODO some pople use 'std::ptr::write_volatile' here
        }
        self._end_write();
    }

    pub fn write_with(&mut self, closure: impl Fn(*mut T)) {
        self._start_write();
        closure(self.item.as_ptr());
        self._end_write();
    }

    fn _start_write(&mut self) {
        assert!(
            self.iteration.load(Ordering::Relaxed) % 2 == 0,
            "single writer allowed"
        );

        self.iteration.fetch_add(1, Ordering::Release); // acquire not required because writer is single threaded.
                                                        // also acquire is available for read only, this is store
    }
    fn _end_write(&mut self) {
        assert!(
            self.iteration.load(Ordering::Relaxed) % 2 == 1,
            "single writer allowed"
        );

        self.iteration.fetch_add(1, Ordering::Release);
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
            while !self.try_read(&mut *val.as_mut_ptr()) {
                std::thread::yield_now();
            }
            *val.as_mut_ptr()
        }
    }

    pub fn try_read(&self, val: &mut T) -> bool {
        let prev = self.iteration.load(Ordering::Acquire);
        if prev % 2 == 0 {
            unsafe {
                *val = *self.item.as_ptr(); // TODO some people use 'std::ptr::read_volatile' here...
            }
            return prev == self.iteration.load(Ordering::Acquire);
        }
        return false;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;
    use super::*;

    fn generate_numbers(start: u64, arr: &mut [u64]) {
        for i in 0..arr.len() {
            arr[i] = start + i as u64;
        }
    }

    fn is_array_increasing(arr: &[u64]) {
        for i in 1..arr.len() {
            if arr[i] - 1 != arr[i - 1] {
                panic!("idx {} not equal {:?} != {:?}", i, arr[i], arr[i - 1]);
            }
        }
    }

    #[test]
    fn test_single_consumer_one_cacheline() {
        type Obj = [u64; 8];
        let my_lock = Arc::new(SeqLock::<Obj>::new([0; 8]));

        // initialize
        let mut value : Obj = [0; 8];
        {
            generate_numbers(0, &mut value);
            let mut writer = my_lock.get_writer();
            writer.write(value.clone());
        }

        let iterations = 100000000;

        let lock_reader = my_lock.clone();
        let reader_thread = thread::spawn(move || {
            let reader = lock_reader.get_reader();
            for _ in 0..iterations {
                let value = reader.read();
                is_array_increasing(&value);
            }
        });

        let lock_writer = my_lock.clone();
        let writer_thread = thread::spawn(move || {
            let mut writer = lock_writer.get_writer();
            for i in 0..iterations {
                generate_numbers(i, &mut value);
                writer.write_with(|item| unsafe { *item = value; });
            }
        });

        reader_thread.join().unwrap();
        writer_thread.join().unwrap();
    }
}

} // mod seqlock
