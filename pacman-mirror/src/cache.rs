use std::sync::{Arc, Mutex};

use replay_buffer::ReplayBuffer;


#[derive(Clone)]
pub enum DataSource {
    Empty,
    Memory(Arc<ReplayBuffer<u8>>),
}

pub struct Cache {
    src: Mutex<DataSource>,
}

impl Cache {
    pub fn new() -> Self {
        Self { src: DataSource::Empty.into() }
    }
    pub fn set(&self, source: DataSource) {
        *self.src.lock().unwrap() = source;
    }
    pub fn get(&self) -> DataSource {
        self.src.lock().unwrap().clone()
    }
}

