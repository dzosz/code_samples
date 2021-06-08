use std::sync::atomic::{AtomicUsize, Ordering};

use std::sync::Arc;

use std::mem::MaybeUninit;

// use std::ptr::NonNull;

//use std::ops::{Deref, DerefMut};

type Counter = AtomicUsize;

use std::thread;

use std::cell::Cell;

#[repr(align(64))]
struct SeqLock<T> {
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

struct SeqLockWriter<'a, T> {
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

struct SeqLockReader<'a, T> {
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
    use super::*;

    fn generate_numbers(start: u32, arr: &mut [u32]) {
        for i in 0..arr.len() {
            arr[i] = start + i as u32;
        }
    }

    fn is_array_increasing(arr: &[u32]) {
        for i in 1..arr.len() {
            if arr[i] - 1 != arr[i - 1] {
                panic!("idx {} not equal {:?} != {:?}", i, arr[i], arr[i - 1]);
            }
        }
    }

    #[test]
    fn test_single_consumer_one_cacheline() {
        type Obj = [u32; 16];
        let my_lock = Arc::new(SeqLock::<Obj>::new([0; 16]));

        // initialize
        let mut value : Obj = [0; 16];
        {
            generate_numbers(0, &mut value);
            let mut writer = my_lock.get_writer();
            writer.write(value.clone());
        }

        let lock_reader = my_lock.clone();
        let reader_thread = thread::spawn(move || {
            let reader = lock_reader.get_reader();
            for _ in 0..100000000 {
                let value = reader.read();
                is_array_increasing(&value);
            }
        });

        let lock_writer = my_lock.clone();
        let writer_thread = thread::spawn(move || {
            let mut writer = lock_writer.get_writer();
            for i in 0..100000000 {
                generate_numbers(i, &mut value);
                writer.write_with(|item| unsafe { *item = value; });
            }
        });

        reader_thread.join().unwrap();
        writer_thread.join().unwrap();
    }
}

fn main() {
    let my_lock = Arc::new(SeqLock::<usize>::new(0));

    let lock_ref1 = my_lock.clone();
    thread::spawn(move || {
        let reader = lock_ref1.get_reader();
        let value = reader.read();
        println!("value={}", value);
    })
    .join()
    .unwrap();

    // overwrite
    let lock_ref2 = my_lock.clone();
    thread::spawn(move || {
        let mut writer = lock_ref2.get_writer();
        writer.write(115);
        writer.write_with(|item| unsafe {
            *item = 111;
        });
        println!("other thread wrote value");
    })
    .join()
    .unwrap();

    // read again
    {
        let reader = my_lock.get_reader();
        let value = reader.read();
        println!("value={}", value);
    }
}
