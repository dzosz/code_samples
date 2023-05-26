extern crate test;

pub mod lockfree_vec {
    use super::test::Bencher;
    use std::process::Termination;
    use crate::lockfree_vec::*;
    use std::sync::{Arc, Mutex};

    const LIMIT : usize = 15360;

    #[bench]
    fn bench_st_lockfreevec_writes(bencher: &mut Bencher) -> impl Termination {
        let vec = LockfreeVec::new();
        vec.reserve(LIMIT);
        let mut iteration : usize = 0;
        bencher.iter(||  { 
            let idx  = iteration % LIMIT;
            vec.write(idx, iteration);
            iteration += 1;

        });
    }

    #[bench]
    fn bench_st_mutex_stdvec_writes(bencher: &mut Bencher) -> impl Termination {
        let mut vec = Vec::new();
        vec.resize(LIMIT, 0);
        let m = Arc::new(Mutex::new(vec));
        let mut iteration : usize = 0;
        bencher.iter(||  { 
            let idx  = iteration % LIMIT;
            m.lock().unwrap()[idx] = iteration;
            iteration += 1;

        });
    }
}
