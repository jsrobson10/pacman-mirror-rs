use std::sync::{Arc, Condvar, Mutex, RwLock};

pub use read::ReplayBufferReader;
pub use write::ReplayBufferWriter;

mod read;
mod write;


struct State {
    num_writing: usize,
    size: usize,
}

pub struct ReplayBuffer<T> where T: Clone {
    data: RwLock<Vec<T>>,
    state: Mutex<State>,
    cvar: Condvar,
}

impl<T> ReplayBuffer<T> where T: Clone {
    pub fn empty() -> Arc<Self> {
        Arc::new(Self {
            data: RwLock::new(Vec::new()),
            state: Mutex::new(State {
                num_writing: 0,
                size: 0,
            }),
            cvar: Condvar::new(),
        })
    }
    pub fn read(self: &Arc<Self>) -> ReplayBufferReader<T> {
        ReplayBufferReader::new(self.clone())
    }
    pub fn write(self: &Arc<Self>) -> ReplayBufferWriter<T> {
        ReplayBufferWriter::new(self.clone())
    }
}

impl ReplayBuffer<u8> {
}

#[cfg(test)]
mod tests {
    use crate::ReplayBuffer;

    #[test]    
    fn multiple_writers() {
        let buf = ReplayBuffer::<()>::empty();
        let w1 = buf.write(); assert!(w1.is_ok());
        let w2 = buf.write(); assert!(w2.is_err());
        drop(w1);
        let w3 = buf.write(); assert!(w3.is_ok());
        let w4 = buf.write(); assert!(w4.is_err());
        let w5 = buf.write(); assert!(w5.is_err());
    }
}
