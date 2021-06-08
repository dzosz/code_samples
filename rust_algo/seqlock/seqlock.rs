use std::sync::atomic::{AtomicUsize, Ordering};

use std::sync::Arc;

use std::mem::MaybeUninit;

use std::ptr::NonNull;

use std::sync::atomic;

//use std::ops::{Deref, DerefMut};

type Counter = AtomicUsize;

use std::thread;

use std::cell::UnsafeCell;

#[repr(align(64))]
struct SeqLock<T> {
    iteration: Counter,
    item: UnsafeCell<T>,
}

struct SeqLockWriter<'a, T> {
    iteration: NonNull<Counter>,
    item: &'a UnsafeCell<T>,
}

impl<T: Copy> SeqLockWriter<'_, T> {
    // single writer  only
    pub fn write(&mut self, val: T) {
        unsafe {
            assert!(
                self.iteration.as_ref().load(Ordering::Relaxed) % 2 == 0,
                "single writer allowed"
            );

            self.iteration.as_mut().fetch_add(1, Ordering::Release); // acquire not required because writer is single threaded.
            // also acquire is available for read only, this is store

            std::ptr::write(self.item.get(), val); // TODO some pople use 'std::ptr::write_volatile' here
            self.iteration.as_mut().fetch_add(1, Ordering::Release);
        }
    }

    pub fn get_ptr(&mut self) -> *mut T {
        self.item.get() as *mut _
    }
}
/*
impl<T> Drop for SeqLockWriter<T> {
    fn drop(&mut self) {
        println!("dropped");
    }
}
*/

struct SeqLockReader<'a, T> {
    iteration: NonNull<Counter>,
    item: &'a UnsafeCell<T>,
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
        unsafe {
            let prev = self.iteration.as_ref().load(Ordering::Acquire);
            if prev % 2 == 0 {
                *val = *self.item.get(); // TODO some people use 'std::ptr::read_volatile' here...
                                         // relaxed because if the count didn't change then underlying buffer is the same as well
                return prev == self.iteration.as_ref().load(Ordering::Relaxed);
            }
            return false;
        }
    }
}

// required to make UnsafeCell safe for multithreaded access
unsafe impl<T> Send for SeqLock<T> {}
unsafe impl<T> Sync for SeqLock<T> {}

impl<T> SeqLock<T> {
    pub fn new(val: T) -> SeqLock<T> {
        SeqLock {
            item: val.into(),
            iteration: AtomicUsize::new(0),
        }
    }

    pub fn get_writer(&mut self) -> SeqLockWriter<T> {
        SeqLockWriter {
            item: &mut self.item,
            iteration: NonNull::new(&self.iteration as *const _ as *mut _).unwrap(),
        }
    }
    pub fn get_reader(&self) -> SeqLockReader<T> {
        SeqLockReader {
            item: &self.item,
            iteration: NonNull::new(&self.iteration as *const _ as *mut _).unwrap(),
        }
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
        writer.write(130);
        writer.write(150);
        let my_ptr = writer.get_ptr();
        let my_ptr2 = writer.get_ptr();
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
