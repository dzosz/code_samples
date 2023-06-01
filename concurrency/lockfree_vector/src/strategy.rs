use crate::descriptor::Descriptor;
//use crate::descriptor::WriteDescriptor;

use crossbeam_epoch;
use crossbeam_queue;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::thread;

pub trait Strategy<T> {
    type GuardT = T;
    //fn update(&self, f: impl Fn(&mut Descriptor));
    fn guard(&self) -> T;
    fn alloc(&self) -> *mut Descriptor;
    fn access(&self, guard: &Self::GuardT) -> *mut Descriptor;
    fn release_access(&self, desc: *mut Descriptor);
    fn dealloc(&self, new_desc: *mut Descriptor, guard: &Self::GuardT);
    fn swap(&self, prev: *mut Descriptor, new_desc: *mut Descriptor, guard: &Self::GuardT) -> bool;
    fn as_ref(&self, guard: &Self::GuardT) -> &Descriptor;
}

// TODO add destructor
thread_local! {
static DESCRIPTOR_CACHE: RefCell<Vec<*mut Descriptor>> = RefCell::new(Vec::with_capacity(8));
}

static DESCRIPTOR_BUFFER: Lazy<crossbeam_queue::ArrayQueue<Box<Descriptor>>> = Lazy::new(|| {
    crossbeam_queue::ArrayQueue::new(64)
});

fn alloc_from_buffer() -> Box<Descriptor> {
    match DESCRIPTOR_BUFFER.pop() {
        Some(item) => item,
        None => Box::new(Descriptor::new(0, None)),
    }
}

fn free_to_buffer(obj: Box<Descriptor>) {
    match DESCRIPTOR_BUFFER.push(obj) {
        Ok(_) => {},
        Err(_) => { // already dropped
        }
    }
}

// FIXME this Reclamation Strategy is not lockfree as it uses Descriptor.counter as a spinlock
// this approach ensures that no ABA problem exists
// TODO replace with handcrafted epoch/hazard pointers?
pub struct SingleReferenceStrategy {
    source: AtomicPtr<Descriptor>,
}

// Epoch based reclamation strategy
// Slow!
pub struct EpochGarbageCollectionStrategy {
    source: crossbeam_epoch::Atomic<Descriptor>,
}

impl EpochGarbageCollectionStrategy {
    pub fn new() -> EpochGarbageCollectionStrategy {
        EpochGarbageCollectionStrategy {
            source: crossbeam_epoch::Atomic::new(Descriptor::new(0, None)),
        }
    }
}

impl Strategy<crossbeam_epoch::Guard> for EpochGarbageCollectionStrategy {
    fn as_ref(&self, guard: &Self::GuardT) -> &Descriptor {
        unsafe { self.access(&guard).as_ref().unwrap() }
    }
    fn guard(&self) -> crossbeam_epoch::Guard {
        return crossbeam_epoch::pin();
    }

    fn alloc(&self) -> *mut Descriptor {
        Box::into_raw(alloc_from_buffer())
    }
    fn access(&self, guard: &Self::GuardT) -> *mut Descriptor {
        self.source.load(Ordering::Relaxed, &guard).as_raw() as *mut Descriptor
    }

    fn release_access(&self, _desc: *mut Descriptor) {}

    fn dealloc(&self, new_desc: *mut Descriptor, guard: &Self::GuardT) {
        let prev = crossbeam_epoch::Shared::from(new_desc as *const Descriptor);
        unsafe {
            guard.defer_unchecked(move || free_to_buffer(prev.into_owned().into_box()));
        }
    }

    fn swap(&self, prev: *mut Descriptor, new_desc: *mut Descriptor, guard: &Self::GuardT) -> bool {
        let new_obj = unsafe { crossbeam_epoch::Owned::from_raw(new_desc) };
        let prev = crossbeam_epoch::Shared::from(prev as *const _);
        let freed = self.source.compare_exchange(
            prev,
            new_obj.into_shared(&guard),
            Ordering::SeqCst,
            Ordering::Relaxed,
            &guard,
        );

        freed.is_ok()
    }
}

impl Strategy<()> for SingleReferenceStrategy {
    //fn update(&self, f: impl Fn(&mut Descriptor)) {
    //    f(self.as_mut());
    //}
    fn guard(&self) {
        return ();
    }

    // allocate from thread local cache
    fn alloc(&self) -> *mut Descriptor {
        let mut reclaimed = std::ptr::null_mut();

        DESCRIPTOR_CACHE.with_borrow_mut(|v| {
            if let Some(idx) = v.iter().position(|ptr| {
                let desc = unsafe { ptr.as_mut().unwrap() };
                return desc
                    .counter
                    .compare_exchange(0, 1, Ordering::SeqCst, Ordering::Relaxed)
                    .is_ok();
            }) {
                reclaimed = v.swap_remove(idx);
            } else {
                // empty cache - allocate
                let boxed = Box::new(Descriptor::new(0, None));
                boxed.counter.store(1, Ordering::Relaxed);
                reclaimed = Box::into_raw(boxed);
            }
        });
        reclaimed
    }

    // spinlock protected access
    // TODO move to the guard and use RAII if possible?
    fn access(&self, _guard: &Self::GuardT) -> *mut Descriptor {
        loop {
            let ptr = self.as_ptr();
            unsafe {
                if ptr
                    .as_ref()
                    .unwrap()
                    .counter
                    .compare_exchange(0, 1, Ordering::SeqCst, Ordering::Relaxed)
                    .is_err()
                {
                    thread::yield_now();
                    continue;
                }
            }
            return ptr;
        }
    }

    fn release_access(&self, desc: *mut Descriptor) {
        unsafe {
            desc.as_ref()
                .unwrap()
                .counter
                .fetch_sub(1, Ordering::SeqCst);
        }
    }

    fn dealloc(&self, new_desc: *mut Descriptor, _guard: &Self::GuardT) {
        DESCRIPTOR_CACHE.with_borrow_mut(|v| {
            v.push(new_desc);
        });

        unsafe {
            new_desc
                .as_mut()
                .unwrap()
                .counter
                .fetch_sub(1, Ordering::SeqCst);
        };
    }

    fn swap(&self, prev: *mut Descriptor, new_desc: *mut Descriptor, _guard: &Self::GuardT) -> bool {
        match self
            .source
            .compare_exchange(prev, new_desc, Ordering::SeqCst, Ordering::Relaxed)
        {
            Ok(_) => return true,
            Err(_) => false,
        }
    }
    fn as_ref(&self, _guard: &Self::GuardT) -> &Descriptor {
        unsafe { self.as_ptr().as_ref().unwrap() }
    }
}

impl SingleReferenceStrategy {
    pub fn new() -> SingleReferenceStrategy {
        SingleReferenceStrategy {
            source: AtomicPtr::new(Box::into_raw(Box::new(Descriptor::new(0, None)))),
        }
    }

    pub fn as_mut(&self) -> &mut Descriptor {
        unsafe { self.as_ptr().as_mut().unwrap() }
    }

    pub fn as_ptr(&self) -> *mut Descriptor {
        self.source.load(Ordering::SeqCst)
    }
}
