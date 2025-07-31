use std::sync::{Arc, Condvar, Mutex, RwLock};

pub use read::ReplayBufferReader;
pub use write::ReplayBufferWriter;

mod read;
mod write;


struct State {
    is_writing: bool,
    size: usize,
}

pub struct ReplayBuffer<T> where T: Clone {
    data: RwLock<Vec<T>>,
    state: Mutex<State>,
    cvar: Condvar,
}

impl<T> ReplayBuffer<T> where T: Clone {
    fn empty() -> Arc<Self> {
        Arc::new(Self {
            data: RwLock::new(Vec::new()),
            state: Mutex::new(State {
                is_writing: true,
                size: 0,
            }),
            cvar: Condvar::new(),
        })
    }
    pub fn read(self: &Arc<Self>) -> ReplayBufferReader<T> {
        ReplayBufferReader::new(self.clone())
    }
}

