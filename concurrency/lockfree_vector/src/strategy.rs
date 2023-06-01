use crate::descriptor::Descriptor;

use crossbeam_epoch;
use crossbeam_queue;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::sync::atomic::{Ordering,AtomicPtr};
use std::thread;

// Guard intentionally is not using RAII because that would hide part of the lockfree algorithm
// Drop on Guard should call release() on used resources
// Ptr should be accessible only through Guard and not be exposed by Strategy
// Simply not a good code
pub struct DescriptionGuard {
    guard: Option<crossbeam_epoch::Guard>,
    //pub ptr: *mut Descriptor,
    //strategy: &'a dyn Strategy<'a, GuardT = DescriptionGuard<'a>>,
}


impl DescriptionGuard {
    fn new(guard : Option<crossbeam_epoch::Guard>) -> Self {
        DescriptionGuard { 
            guard,
        }
    }

    //pub fn as_ref(&self) -> &Descriptor {
    //    unsafe { self.ptr.as_ref().unwrap() }
    //}
}

pub trait Strategy {
    // type GuardT = DescriptionGuard;
    //fn update(&self, f: impl Fn(&mut Descriptor));
    fn guard(&self) -> DescriptionGuard;
    fn alloc(&self) -> *mut Descriptor;
    fn access(&self, guard: &DescriptionGuard) -> *mut Descriptor;
    fn release_access(&self, desc: *mut Descriptor);
    fn dealloc(&self, new_desc: *mut Descriptor, guard: &DescriptionGuard);
    fn swap(&self, prev: *mut Descriptor, new_desc: *mut Descriptor, guard: &DescriptionGuard) -> bool;
    fn descriptor(&self, guard: &DescriptionGuard) -> &Descriptor;
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

impl Strategy for EpochGarbageCollectionStrategy {
    fn descriptor(&self, guard: &DescriptionGuard) -> &Descriptor {
        unsafe { self.access(&guard).as_ref().unwrap() }
    }

    fn guard(&self) -> DescriptionGuard {
        return DescriptionGuard::new(Some(crossbeam_epoch::pin()));

    }

    fn alloc(&self) -> *mut Descriptor {
        Box::into_raw(alloc_from_buffer())
    }

    fn access(&self, guard: &DescriptionGuard) -> *mut Descriptor {
        self.source.load(Ordering::Relaxed, &guard.guard.as_ref().unwrap()).as_raw() as *mut Descriptor
    }

    fn release_access(&self, _desc: *mut Descriptor) {}

    fn dealloc(&self, new_desc: *mut Descriptor, guard: &DescriptionGuard) {
        let prev = crossbeam_epoch::Shared::from(new_desc as *const Descriptor);
        unsafe {
            guard.guard.as_ref().unwrap().defer_unchecked(move || free_to_buffer(prev.into_owned().into_box()));
        }
    }

    fn swap(&self, prev: *mut Descriptor, new_desc: *mut Descriptor, guard: &DescriptionGuard) -> bool {
        let new_obj = unsafe { crossbeam_epoch::Owned::from_raw(new_desc) };
        let prev = crossbeam_epoch::Shared::from(prev as *const _);
        let freed = self.source.compare_exchange(
            prev,
            new_obj.into_shared(&guard.guard.as_ref().unwrap()),
            Ordering::SeqCst,
            Ordering::Relaxed,
            &guard.guard.as_ref().unwrap(),
        );

        freed.is_ok()
    }
}


// FIXME this Reclamation Strategy is not lockfree as it uses Descriptor.counter as a spinlock
// this approach ensures that no ABA problem exists
// TODO replace with handcrafted epoch/hazard pointers?
pub struct SpinlockDescriptorStrategy {
    source: AtomicPtr<Descriptor>,
}

impl Strategy for SpinlockDescriptorStrategy {
    fn guard(&self) -> DescriptionGuard {
        return DescriptionGuard::new(None);
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
    fn access(&self, _guard: &DescriptionGuard) -> *mut Descriptor {
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

    fn dealloc(&self, new_desc: *mut Descriptor, _guard: &DescriptionGuard) {
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

    fn swap(&self, prev: *mut Descriptor, new_desc: *mut Descriptor, _guard: &DescriptionGuard) -> bool {
        match self
            .source
            .compare_exchange(prev, new_desc, Ordering::SeqCst, Ordering::Relaxed)
        {
            Ok(_) => return true,
            Err(_) => false,
        }
    }
    fn descriptor(&self, _guard: &DescriptionGuard) -> &Descriptor {
        unsafe { self.as_ptr().as_ref().unwrap() }
    }
}

impl SpinlockDescriptorStrategy {
    pub fn new() -> SpinlockDescriptorStrategy {
        SpinlockDescriptorStrategy {
            source: AtomicPtr::new(Box::into_raw(Box::new(Descriptor::new(0, None)))),
        }
    }

    pub fn as_ptr(&self) -> *mut Descriptor {
        self.source.load(Ordering::SeqCst)
    }
}
