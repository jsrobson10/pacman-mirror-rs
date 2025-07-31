use std::{io::Read, sync::Arc};

use super::ReplayBuffer;


pub struct ReplayBufferReader<T> where T: Clone {
    base: Arc<ReplayBuffer<T>>,
    at: usize,
}

impl<T> ReplayBufferReader<T> where T: Clone {
    pub fn new(base: Arc<ReplayBuffer<T>>) -> Self {
        Self { base, at: 0 }
    }
    fn wait_for(&self, count: usize) {
        let target = self.at + count;
        let mut lock = self.base.state.lock().unwrap();
        while lock.num_writing > 0 && lock.size < target {
            lock = self.base.cvar.wait(lock).unwrap();
        }
    }
}

impl<T> Iterator for ReplayBufferReader<T> where T: Clone {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.wait_for(1);
        self.base.data.read().unwrap().get(self.at)
            .map(|v| { self.at += 1; v.clone() })
    }
}

impl Read for ReplayBufferReader<u8> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.wait_for(buf.len());
        let lock = self.base.data.read().unwrap();
        let count = buf.iter_mut()
            .zip(lock.iter().copied())
            .map(|(dst, src)| *dst = src)
            .count();
        self.at += count;
        Ok(count)
    }
}

