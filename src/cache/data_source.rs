use std::sync::{mpsc, Arc};


#[derive(Clone)]
pub enum DataSource {
    Empty,
    Partial(mpsc::Sender<os_pipe::PipeWriter>),
    Memory(Arc<[u8]>),
}

