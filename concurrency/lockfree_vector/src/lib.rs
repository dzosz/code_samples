#![feature(vec_into_raw_parts)]
#![feature(associated_type_defaults)]
#![feature(atomic_from_ptr, atomic_from_mut, pointer_is_aligned)]
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

    fn highest_bit_index(num : usize) -> usize {
        //1 << (usize::BITS - num.leading_zeros()) >> 1;
        //((1 << (usize::BITS - num.leading_zeros()) >> 1) as usize).trailing_zeros() as usize
        ((usize::BITS - num.leading_zeros()) - 1) as usize // 0 unsupported
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
        memory: [*mut * mut usize; 64], // pointer to buckets, TODO generic!
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
                memory: [std::ptr::null_mut(); 64]
                //field: Default::default(),
            }
        }

        pub fn push_back(&self, elem: DataT) {
            dbg!("push_back elem={}");
            loop {
                let desc = self.descriptor.load(Ordering::SeqCst);
                self.complete_write(desc);
                let desc_ref = unsafe { desc.as_mut().unwrap() };
                dbg!("push_back elem={} dsize=", elem, desc_ref.size);
                {
                    let bucket = highest_bit_index(desc_ref.size + FIRST_BUCKET_SIZE) - highest_bit_index(FIRST_BUCKET_SIZE);
                    let bucket_ptr = self.get_bucket(bucket).load(Ordering::SeqCst);
                    if bucket_ptr.is_null() {
                         self.alloc_bucket(bucket);
                    }
                }
                let writeop = WriteDescriptor::new(self.read(desc_ref.size), elem, desc_ref.size);
                let new_desc = Box::into_raw(Box::new(Descriptor::new(desc_ref.size+1, Some(writeop))));
                match self.descriptor.compare_exchange(desc, new_desc, Ordering::SeqCst, Ordering::Relaxed) {
                    Ok(_) =>  {
                        self.complete_write(new_desc);
                        return;
                    },
                    _ =>  unsafe { 
                        dbg!("err push back");
                        drop(Box::from_raw(new_desc));
                    }
                }
            }
        }

        pub fn pop_back(&self) -> Option<DataT> {
            loop {
                let desc = self.descriptor.load(Ordering::SeqCst);
                self.complete_write(desc);
                let desc_ref = unsafe { desc.as_mut().unwrap() };
                //debug_assert!(desc_ref.size > 0, "pop_back must not be called on empty vec");
                if  desc_ref.size == 0 {
                    return None
                }
                let elem = self.read(desc_ref.size - 1);
                let new_desc = Box::new(Descriptor::new(desc_ref.size - 1, None));
                let new_desc_ptr = Box::into_raw(new_desc);
                match self.descriptor.compare_exchange(desc, new_desc_ptr, Ordering::SeqCst, Ordering::Relaxed) {
                    Ok(_) => return Some(elem), 
                    _ =>  unsafe { drop(Box::from_raw(new_desc_ptr)); }
                }
            }
        }

        pub fn read(&self, i : usize) -> DataT {
            dbg!("read {}", i);
            self.at(i).load(Ordering::SeqCst) // DataT::from(...)
        }

        pub fn write(&self, i : usize, elem: DataT) {
            dbg!("write {}", i);
            self.at(i).store(elem, Ordering::SeqCst);
        }

        pub fn reserve(&self, size : usize) {
            let mut i = unsafe { highest_bit_index(self.descriptor.load(Ordering::SeqCst).as_ref().unwrap().size + FIRST_BUCKET_SIZE -1) - highest_bit_index(FIRST_BUCKET_SIZE) };
            i = cmp::max(i, 0);

            while i < highest_bit_index(size + FIRST_BUCKET_SIZE -1) - highest_bit_index(FIRST_BUCKET_SIZE) {
                i = i + 1;
                self.alloc_bucket(i);
            }
        }

        pub fn size(&self) -> usize {
            let desc = unsafe {self.descriptor.load(Ordering::SeqCst).as_ref().unwrap() };
            match desc.pending {
                Some(ref writeop) => desc.size - (!writeop.completed as usize),
                _ => desc.size
            }
        }

        fn at(&self, i : usize) -> &AtomicUsize {
            let (bucket, idx) = Self::index_to_bucket_and_pos(i);
            let bucket_ptr = self.get_bucket(bucket).load(Ordering::SeqCst);
            let item_ptr =  unsafe { bucket_ptr.offset(idx as isize)};
            dbg!("bucket_ptr {} offseted ", item_ptr);
            //dbg!("bucket_ptr {} offseted ", unsafe {bucket_ptr.as_ref().unwrap()});
            // TODO this is nullXYZ 
            //let item = unsafe { AtomicUsize::from_mut(&mut **bucket_ptr) };
            let item = unsafe { AtomicUsize::from_ptr(item_ptr as * mut usize) };
            //let item = unsafe { AtomicUsize::from_ptr(bucket_ptr.as_ref().unwrap().offset(idx as isize)) };
            item
        }

        fn complete_write(&self, desc : *mut Descriptor ) {
            dbg!("complete_write {}");
            unsafe {
                if let Some(ref mut writeop) = (*desc).pending.as_mut() {
                    if !writeop.completed {
                        let ref value = self.at(writeop.pos);
                        match value.compare_exchange(writeop.old_value, writeop.new_value, Ordering::SeqCst, Ordering::Relaxed) {
                            Ok(_) => {
                                dbg!("wrote {} {} {}", writeop.old_value, writeop.new_value, writeop.pos);
                                writeop.completed = true;
                            },
                            _ => {
                                // different thread succeeded
                            }
                        }

                    }
                }
            }
        }

        fn alloc_bucket(&self, bucket: usize) {
            let bucket_size = FIRST_BUCKET_SIZE.pow(bucket as u32 + 1); // TODO bitshift
            let mem = Vec::with_capacity(bucket_size);
            dbg!("alloc {} size={}", bucket, bucket_size);
            let (mem_ptr, _, _) = mem.into_raw_parts();
            let null = std::ptr::null_mut();
            let bucket_ptr = self.get_bucket(bucket);
            match bucket_ptr.compare_exchange(null, mem_ptr, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(_) => {} ,
                Err(_) => unsafe { dbg!("alloc err"); drop(Vec::from_raw_parts(mem_ptr, bucket_size, 0)); }
            }
        }

        fn get_bucket(&self, bucket: usize) -> & AtomicPtr<*mut usize> {
            dbg!("get_bucket {} {} ", bucket, self.memory[bucket]);
            let addr = std::ptr::addr_of!(self.memory[bucket]);
            unsafe { AtomicPtr::from_ptr(addr as * mut * mut *mut usize) }
            //unsafe { AtomicPtr::from_mut(&mut self.memory[bucket]) }
            //unsafe { AtomicPtr::from_ptr(self.memory[bucket]) }
        }

        fn index_to_bucket_and_pos(i: usize) -> (usize, usize) {
            let pos = i + FIRST_BUCKET_SIZE; // 8 + 8 = 0b1000
            let hibit = highest_bit_index(pos); //  5 should be 3?
            let idx = pos ^ (1 << hibit); //  0b1000 ^ (1 << 3) = 0
            let bucket = hibit - highest_bit_index(FIRST_BUCKET_SIZE);
            dbg!("at i {} bucket {} idx {} hibit {} pos {} ", i, bucket, idx, hibit, pos);
            return (bucket, idx);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::convert::TryInto;
        use std::sync::Arc;
        use std::thread;

        #[test]
        fn test_indexing() {
            assert_eq!((1,0), LockfreeVec::index_to_bucket_and_pos(8));
            assert_eq!((0,0), LockfreeVec::index_to_bucket_and_pos(0));
            assert_eq!((0,1), LockfreeVec::index_to_bucket_and_pos(1));
            assert_eq!((0,2), LockfreeVec::index_to_bucket_and_pos(2));
            assert_eq!((0,3), LockfreeVec::index_to_bucket_and_pos(3));
            assert_eq!((0,7), LockfreeVec::index_to_bucket_and_pos(7));
            assert_eq!((1,0), LockfreeVec::index_to_bucket_and_pos(8));
            assert_eq!((1,8), LockfreeVec::index_to_bucket_and_pos(16));
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
        fn test_push_back() {
            let vec = LockfreeVec::new();
            for i in 0..640 {
                vec.push_back(i);
                assert_eq!(vec.read(i), i);
            }
        }

        #[test]
        fn test_pop_back() {
            let vec = LockfreeVec::new();
            for i in 0..640 {
                vec.push_back(i);
            }
            for i in 640..0 {
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
            let iterations = 640;
            for i in 0..iterations {
                vec.push_back(i);
            }
            for i in 0..iterations {
                assert_eq!(vec.read(i), i);
            }
        }

        #[test]
        fn test_concurrent_writes() {
            let vec = Arc::new(LockfreeVec::new());
            let vec2 = vec.clone();
            let iterations = 20000;
            let writer_thread = thread::spawn(move || {
                for i in 0..iterations {
                    vec2.push_back(i);
                }
            });

            {
                let mut verify_vec = Vec::new();
                for _ in 0..iterations {
                    loop {
                        //if vec.size() > 0
                        {
                            let item = vec.pop_back();

                            //assert_eq!(item.is_some(), true);
                            if item.is_some() {
                                verify_vec.push(item.unwrap());
                                break;
                            }
                        }
                    }
                }
                verify_vec.sort();
                assert_eq!(verify_vec.len(), iterations);
                for i in 0..iterations {
                    assert_eq!(verify_vec[i], i);
                }
            }
            assert_eq!(vec.size(), 0);
            writer_thread.join().unwrap();
        }

    }

} // mod 


