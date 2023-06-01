use std::sync::atomic::{AtomicUsize,AtomicBool};

type Counter = AtomicUsize;
type DataT = usize; // TODO should be generic

pub struct WriteDescriptor {
    pub old_value: DataT,
    pub new_value: DataT,
    pub pos: usize, // pos in memory array
    pub completed: AtomicBool,
}

impl WriteDescriptor {
    pub fn new(old: DataT, new: DataT, p: usize) -> Self {
        WriteDescriptor {
            old_value: old,
            new_value: new,
            pos: p,
            completed: AtomicBool::new(false),
        }
    }
}

pub struct Descriptor {
    pub size: usize,
    pub counter: Counter, // used for reference counting reclaim strategy
    pub pending: Option<WriteDescriptor>,
}

impl Descriptor {
    pub fn new(s: usize, pen: Option<WriteDescriptor>) -> Self {
        Descriptor {
            size: s,
            counter: Counter::new(0),
            pending: pen,
        }
    }
}
