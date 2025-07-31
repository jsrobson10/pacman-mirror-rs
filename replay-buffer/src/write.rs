use std::{io::Write, sync::Arc};

use super::ReplayBuffer;


pub struct ReplayBufferWriter<T> where T: Clone {
    base: Arc<ReplayBuffer<T>>,
}

impl<T> ReplayBufferWriter<T> where T: Clone {
    pub fn new() -> Self {
        Self { base: ReplayBuffer::empty() }
    }
    pub fn push(&self, value: T) {
        let mut data = self.base.data.write().unwrap();
        let mut state = self.base.state.lock().unwrap();
        data.push(value);
        state.size = data.len();
        self.base.cvar.notify_all();
    }
    pub fn extend(&self, iter: impl IntoIterator<Item = T>) {
        let mut data = self.base.data.write().unwrap();
        let mut state = self.base.state.lock().unwrap();
        data.extend(iter);
        state.size = data.len();
        self.base.cvar.notify_all();
    }
    pub fn source(&self) -> &Arc<ReplayBuffer<T>> {
        &self.base
    }
}

impl Write for ReplayBufferWriter<u8> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.extend(buf.iter().copied());
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<T> Drop for ReplayBufferWriter<T> where T: Clone {
    fn drop(&mut self) {
        self.base.state.lock().unwrap().is_writing = false;
        self.base.cvar.notify_all();
    }
}

