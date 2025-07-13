use std::{io::Write, sync::Arc};

use super::Source;


pub struct Writer {
    src: Arc<Source>,
}

impl Writer {
    pub fn new() -> Writer {
        Writer { src: Source::new() }
    }
    pub fn source(&self) -> &Arc<Source> {
        &self.src
    }
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut data = self.src.data.write().unwrap();
        data.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        let data = self.src.data.read().unwrap();
        let mut state = self.src.state.mtx.lock().unwrap();
        state.len = data.len();
        self.src.state.cvar.notify_all();
        Ok(())
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        let data = self.src.data.read().unwrap();
        let mut state = self.src.state.mtx.lock().unwrap();
        state.len = data.len();
        state.done = true;
        self.src.state.cvar.notify_all();
    }
}

