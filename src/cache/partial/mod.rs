use std::sync::{Arc, RwLock};

mod reader;
mod writer;
mod state;

use state::StateHolder;
pub use reader::Reader;
pub use writer::Writer;


pub struct Source {
    pub data: RwLock<Vec<u8>>,
    state: StateHolder,
}

impl Source {
    fn new() -> Arc<Source> {
        Arc::new(Source {
            data: Vec::new().into(),
            state: StateHolder::new(),
        })
    }
    pub fn reader(self: &Arc<Self>) -> Reader {
        Reader::new(self.clone())
    }
}

