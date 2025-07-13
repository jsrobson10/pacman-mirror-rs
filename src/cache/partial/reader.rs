use std::{io::Read, sync::Arc};

use super::Source;


pub struct Reader {
    src: Arc<Source>,
    at: usize,
}

impl Reader {
    pub fn new(src: Arc<Source>) -> Reader {
        Reader { src, at: 0 }
    }
}

impl Read for Reader {
    fn read(&mut self, buf_dst: &mut [u8]) -> std::io::Result<usize> {
        let data_src = {
            let _lock = self.src.state.wait_and_lock(self.at + 1);
            self.src.data.read().unwrap()
        };
        let len = data_src[self.at..]
            .iter()
            .copied()
            .zip(buf_dst.iter_mut())
            .map(|(src, dst)| *dst = src)
            .count();
        self.at += len;
        Ok(len)
    }
}

