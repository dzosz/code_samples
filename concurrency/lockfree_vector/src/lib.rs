#![feature(vec_into_raw_parts)]
#![feature(associated_type_defaults)]
#![feature(atomic_from_ptr, atomic_from_mut, pointer_is_aligned)]
#![feature(test)]

mod bench;

pub mod lockfree_vec {

    macro_rules! debug {
    ($($e:expr),+) => {
        {
            #[cfg(debug_assertions)]
            {
                dbg!($($e),+)
            }
            #[cfg(not(debug_assertions))]
            {
                ($($e),+)
            }
        }
    };
    }

    const FIRST_BUCKET_SIZE : usize = 8; // must be mutliple of 2

    use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};

    // use std::ptr::NonNull;
    //use std::ops::{Deref, DerefMut};

    type Counter = AtomicUsize;
    type DataT   = usize; // TODO should be generic over vector

    fn highest_bit_index(num : usize) -> usize {
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

    /* TODO
       Initializing arrays requires Copy trait which is not implemented for AtomicPtr. This doesn't matter as long as pointers and data is accessed in atomic way (using AtomicPtr structure in this implementation).
    */
    /* TODO
     * make LockFreeVec generic over DataT
     */

    #[repr(align(64))]
    pub struct LockfreeVec {
        descriptor: AtomicPtr<Descriptor>,
        memory: [*mut * mut usize; 64], // pointer to buckets, TODO generic!
        //field : PhantomData<T>,
    }

    // make safe for multithreaded access
    unsafe impl Send for LockfreeVec {}
    unsafe impl Sync for LockfreeVec {}

    impl Drop for LockfreeVec {
        fn drop(&mut self) {
            unsafe {
                drop(Box::from_raw(self.descriptor.load(Ordering::SeqCst)));
                for bucket in 0..self.memory.len() {
                    let bucket_ptr = self.get_bucket(bucket).load(Ordering::SeqCst);
                    if bucket_ptr.is_null() {
                        break; // next buckets are guaranteed to be free
                    }
                    let bucket_size = 1 << (highest_bit_index(FIRST_BUCKET_SIZE) + bucket);
                    drop(Vec::from_raw_parts(bucket_ptr, bucket_size, bucket_size));
                }
            }
        }
    }

    impl LockfreeVec
    {
        pub fn new() -> LockfreeVec {
            assert_eq!(FIRST_BUCKET_SIZE % 2, 0);

            let empty = Box::new(Descriptor::new(0, None));
            LockfreeVec {
                descriptor: AtomicPtr::new(Box::into_raw(empty)),
                memory: [std::ptr::null_mut(); 64]
                //field: Default::default(),
            }
        }

        pub fn push_back(&self, elem: DataT) {
            debug!("push_back elem={}");
            loop {
                let desc = self.descriptor.load(Ordering::SeqCst);
                self.complete_write(desc);
                let desc_ref = unsafe { desc.as_mut().unwrap() };
                debug!("push_back elem={} dsize=", elem, desc_ref.size);
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
                        debug!("err push back");
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
            debug!("read {}", i);
            debug_assert!(i < self.size());
            self.at(i).load(Ordering::Acquire) // DataT::from(...)
        }

        pub fn write(&self, i : usize, elem: DataT) {
            debug!("write {}", i);
            debug_assert!(i < self.size());
            self.at(i).store(elem, Ordering::Release);
        }

        pub fn reserve(&self, size : usize) {
            let cur_size = unsafe { self.descriptor.load(Ordering::SeqCst).as_ref().unwrap().size };
            let mut i = unsafe {
                    highest_bit_index(cur_size + FIRST_BUCKET_SIZE - (cur_size > 0) as usize)
                - highest_bit_index(FIRST_BUCKET_SIZE) };
            let (bucket, idx) = Self::get_bucket_and_pos_at(size-1);
            debug!("reserve cursize {} i {} bucket={} idx={}", cur_size, i, bucket, idx);
            while i <= bucket {
                self.alloc_bucket(i);
                i = i + 1;
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
            let (bucket, idx) = Self::get_bucket_and_pos_at(i);
            let bucket_ptr = self.get_bucket(bucket).load(Ordering::Acquire);
            let item_ptr =  unsafe { bucket_ptr.offset(idx as isize)};
            debug!("bucket_ptr {} offseted ", item_ptr);
            let item = unsafe { AtomicUsize::from_ptr(item_ptr as * mut usize) };
            item
        }

        fn complete_write(&self, desc : *mut Descriptor ) {
            debug!("complete_write {}");
            unsafe {
                if let Some(ref mut writeop) = (*desc).pending.as_mut() {
                    if !writeop.completed {
                        let ref value = self.at(writeop.pos);
                        match value.compare_exchange(writeop.old_value, writeop.new_value, Ordering::SeqCst, Ordering::Relaxed) {
                            Ok(_) => {
                                debug!("wrote {} {} {}", writeop.old_value, writeop.new_value, writeop.pos);
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
            debug!("alloc bucket={}");
            let bucket_ptr = self.get_bucket(bucket);
            if !bucket_ptr.load(Ordering::Relaxed).is_null() {
                return;
            }

            let bucket_size = 1 << (highest_bit_index(FIRST_BUCKET_SIZE) + bucket);
            let mem = Vec::with_capacity(bucket_size);
            debug!("alloc size={}", bucket, bucket_size);
            let (mem_ptr, _, _) = mem.into_raw_parts();
            let null = std::ptr::null_mut();
            match bucket_ptr.compare_exchange(null, mem_ptr, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(_) => {} ,
                Err(_) => unsafe {
                    // different thread succeeded
                    drop(Vec::from_raw_parts(mem_ptr, bucket_size, bucket_size));
                }
            }
        }

        fn get_bucket(&self, bucket: usize) -> & AtomicPtr<*mut usize> {
            //debug!("get_bucket {} {} ", bucket, self.memory[bucket]);
            let addr = std::ptr::addr_of!(self.memory[bucket]);
            unsafe { AtomicPtr::from_ptr(addr as * mut * mut *mut usize) }
            //unsafe { AtomicPtr::from_mut(&mut self.memory[bucket]) }
            //unsafe { AtomicPtr::from_ptr(self.memory[bucket]) }
        }

        fn get_bucket_and_pos_at(i: usize) -> (usize, usize) {
            let pos = i + FIRST_BUCKET_SIZE; // 8 + 8 = 0b1000
            let hibit = highest_bit_index(pos); //  5 should be 3?
            let idx = pos ^ (1 << hibit); //  0b1000 ^ (1 << 3) = 0
            let bucket = hibit - highest_bit_index(FIRST_BUCKET_SIZE);
            //debug!("at i {} bucket {} idx {} hibit {} pos {} ", i, bucket, idx, hibit, pos);
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
            assert_eq!((0,FIRST_BUCKET_SIZE-1), LockfreeVec::get_bucket_and_pos_at(FIRST_BUCKET_SIZE-1));
            assert_eq!((0,0), LockfreeVec::get_bucket_and_pos_at(0));
            assert_eq!((0,1), LockfreeVec::get_bucket_and_pos_at(1));
            assert_eq!((0,2), LockfreeVec::get_bucket_and_pos_at(2));
            assert_eq!((0,3), LockfreeVec::get_bucket_and_pos_at(3));
            assert_eq!((0,7), LockfreeVec::get_bucket_and_pos_at(7));
            assert_eq!((1,0), LockfreeVec::get_bucket_and_pos_at(FIRST_BUCKET_SIZE));
            assert_eq!((1,FIRST_BUCKET_SIZE), LockfreeVec::get_bucket_and_pos_at(FIRST_BUCKET_SIZE*2));
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
            for i in 0..512 {
                vec.push_back(i);
                assert_eq!(vec.read(i), i);
            }
        }

        #[test]
        fn test_pop_back() {
            let vec = LockfreeVec::new();
            for i in 0..512 {
                vec.push_back(i);
            }
            for i in 512..0 {
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
            vec.reserve(512);
            for i in 0..iterations {
                vec.write(i, i);
            }
            for i in 0..iterations {
                assert_eq!(vec.read(i), i);
            }
        }

        #[test]
        fn test_reserve() { // TODO parametrize this test
            let vec = LockfreeVec::new();

            let check_bucket = |new_size| {
                vec.reserve(new_size);
                let (bucket, _) =  LockfreeVec::get_bucket_and_pos_at(new_size-1);
                assert_eq!(vec.get_bucket(bucket).load(Ordering::Relaxed).is_null(), false);
                assert_eq!(vec.get_bucket(bucket+1).load(Ordering::Relaxed).is_null(), true);
            };

            check_bucket(7);
            check_bucket(8);
            check_bucket(9);
            check_bucket(100);
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
            }
            assert_eq!(vec.size(), 0);
            writer_thread.join().unwrap();
        }

    }

} // mod 


