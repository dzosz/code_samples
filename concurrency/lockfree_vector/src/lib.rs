#![feature(vec_into_raw_parts)]
#![feature(associated_type_defaults)]
pub mod lockfree_vec {

    const FIRST_BUCKET_SIZE : usize = 8;

    use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering, fence};

    use std::mem::MaybeUninit;

    use std::cell::Cell;
    use std::marker::PhantomData;
    use std::cmp;

    // use std::ptr::NonNull;
    //use std::ops::{Deref, DerefMut};

    type Counter = AtomicUsize;
    type DataT   = usize; // TODO should be generic over vector

    fn HighestBit(num : usize) -> usize {
        return 1 << (usize::BITS - num.leading_zeros());
    }

    struct WriteDescriptor {
        old_value: DataT,
        new_value: DataT,
        pos: usize, // pos in memory array
        completed: bool
    }

    impl WriteDescriptor {
        fn new(old: DataT, new: DataT, p: usize) -> Self {
            WriteDescriptor {
                old_value: old,
                new_value: new,
                pos: p,
                completed: false
            }
        }
    }

    struct Descriptor {
        size: usize,
        counter: Counter,
        pending: Option<WriteDescriptor>,
    }

    impl Descriptor {
        pub fn new(s : usize, pen : Option<WriteDescriptor>) -> Self {
            Descriptor { 
                size: s,
                counter: Counter::new(0),
                pending: pen
            }
        }
    }


    /*
    trait MyAtomItem<T : From<usize>> {
        type AtomicT = AtomicUsize;
        type DataT = usize;
    }
    */

    // generic trait
    // trait AtomicVecItem { }
    // impl AtomicVecItem for AtomicUsize {}
    // struct Atomic<T: Atomize>(T::Atom);

    // #[repr(align(64))]
    pub struct LockfreeVec {
        descriptor: AtomicPtr<Descriptor>,
        memory: [AtomicPtr<Vec<AtomicUsize>>; 64], // TODO generic!
        //field : PhantomData<T>,
    }

    // required to make Cell safe for multithreaded access
    unsafe impl Send for LockfreeVec {}
    unsafe impl Sync for LockfreeVec {}

    impl LockfreeVec
        // where T: Copy + From<usize>
    {
        pub fn new() -> LockfreeVec {
            let empty = Box::new(Descriptor::new(0, None));
            LockfreeVec {
                descriptor: AtomicPtr::new(Box::into_raw(empty)),
                memory: [AtomicPtr::new(std::ptr::null_mut()); 64]
                //field: Default::default(),
            }
        }

        pub fn push_back(&self, elem: DataT) {
            loop {
                let desc = self.descriptor.load(Ordering::SeqCst);
                self.complete_write(desc);
                let desc_ref = unsafe { desc.as_mut().unwrap() };
                let bucket = HighestBit(desc_ref.size + FIRST_BUCKET_SIZE) - HighestBit(FIRST_BUCKET_SIZE);
                let bucket_ptr = self.memory[bucket].load(Ordering::SeqCst);
                if bucket_ptr.is_null() {
                     self.alloc_bucket(bucket);
                }
                let writeop = WriteDescriptor::new(self.read(desc_ref.size), elem, desc_ref.size);
                let new_desc = Box::new(Descriptor::new(desc_ref.size+1, Some(writeop)));
                let new_desc_ptr = Box::into_raw(new_desc);
                match self.descriptor.compare_exchange(desc, new_desc_ptr, Ordering::SeqCst, Ordering::Relaxed) {
                    Ok(_) =>  {
                        self.complete_write(new_desc_ptr);
                        return;
                    },
                    _ =>  unsafe { Box::from_raw(new_desc_ptr); } // need to dealloc
                }
            }
        }

        pub fn pop_back(&self) -> Option<DataT> {
            loop {
                let desc = self.descriptor.load(Ordering::SeqCst);
                self.complete_write(desc);
                let desc_ref = unsafe { desc.as_mut().unwrap() };
                let elem = self.read(desc_ref.size - 1);
                let new_desc = Box::new(Descriptor::new(desc_ref.size - 1, None));
                let new_desc_ptr = Box::into_raw(new_desc);
                match self.descriptor.compare_exchange(desc, new_desc_ptr, Ordering::SeqCst, Ordering::Relaxed) {
                    Ok(_) => return Some(elem), 
                    _ =>  unsafe { Box::from_raw(new_desc_ptr); } // need to dealloc
                }
            }
        }

        pub fn read(&self, i : usize) -> DataT {
            self.at(i).load(Ordering::SeqCst) // DataT::from(...)
        }

        pub fn write(&self, i : usize, elem: DataT) {
            self.at(i).store(elem, Ordering::SeqCst);
        }

        pub fn reserve(&self, size : usize) {
            let mut i = unsafe { HighestBit(self.descriptor.load(Ordering::SeqCst).as_ref().unwrap().size + FIRST_BUCKET_SIZE -1) - HighestBit(FIRST_BUCKET_SIZE) };
            i = cmp::max(i, 0);

            while i < HighestBit(size + FIRST_BUCKET_SIZE -1) - HighestBit(FIRST_BUCKET_SIZE) {
                i = i + 1;
                self.alloc_bucket(i);
            }
        }


        pub fn at(&self, i : usize) -> &AtomicUsize {
            let pos = i + FIRST_BUCKET_SIZE;
            let hibit = HighestBit(pos);
            let idx = pos ^ (1 << hibit);
            unsafe {
                self.memory[hibit - HighestBit(FIRST_BUCKET_SIZE)].load(Ordering::SeqCst).as_ref().unwrap().get_unchecked(idx)
            }
        }

        fn complete_write(&self, desc : *mut Descriptor ) {
            unsafe {
                if let Some(ref mut writeop) = (*desc).pending {
                    //if writeop.is_null() {
                    //    return;
                    //}
                    if !writeop.completed {
                        let value = self.at(writeop.pos);
                        let _ = value.compare_exchange(writeop.old_value, writeop.new_value, Ordering::SeqCst, Ordering::Relaxed);
                        writeop.completed = true;
                    }
                }
            }
        }

        fn alloc_bucket(&self, bucket: usize) {
            let idx = (bucket + 1) as u32;
            let bucket_size = FIRST_BUCKET_SIZE.pow(idx); // TODO bitshift
            let mem = Vec::with_capacity(bucket_size);
            let (mem_ptr, _, _) = mem.into_raw_parts();
            let null = std::ptr::null_mut();
            match self.memory[bucket].compare_exchange(null, mem_ptr, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(_) => return,
                Err(_) => unsafe { let _ = Vec::from_raw_parts(mem_ptr, bucket_size, 0); }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::convert::TryInto;
        use std::sync::Arc;
        use std::thread;

    }

} // mod 


