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

    pub fn write_with(&mut self, closure: fn(*mut T)) {
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

// FIXME can't return SeqLockReader/Writer here. also Generic Associated Types are not supported
/*
impl< T> Deref for SeqLock<T> {
    type Target = SeqLockReader<T>;
    fn deref(&self) -> &Self::Target {
        &self
        /*
        SeqLockReader {
            item: &self.item,
            iteration: &self.iteration,
        }*/
    }
}
impl<T> DerefMut for SeqLock<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self
        /*
        SeqLockWriter {
            item: &mut self.item,
            iteration: &self.iteration,
        }*/
    }
}*/

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
        writer.write(150);
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


