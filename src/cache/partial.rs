use std::{io::Write, mem::ManuallyDrop, sync::{mpsc, Arc}};


struct Receiver {
	pipe: os_pipe::PipeWriter,
	at: usize,
}

pub struct PartialCacheWriter {
    rx: mpsc::Receiver<os_pipe::PipeWriter>,
    pipes: Vec<Receiver>,
    data: Vec<u8>,
}

impl PartialCacheWriter {
    pub fn new(rx: mpsc::Receiver<os_pipe::PipeWriter>) -> Self {
        Self { rx, pipes: Vec::new(), data: Vec::new() }
    }
    pub fn add_pipe(&mut self, pipe: os_pipe::PipeWriter) {
        self.pipes.push(Receiver { pipe, at: 0 });
    }
    pub fn write(&mut self, buf: &[u8]) -> usize {
        self.data.extend_from_slice(buf);
        self.pipes.extend(self.rx.try_iter().map(|pipe| {
            Receiver { pipe, at: 0 }
        }));
        self.pipes.retain_mut(|r| {
            r.pipe.write(&self.data[r.at..]).inspect(|len| {
                r.at += len;
            }).is_ok()
        });
        buf.len()
    }
    pub fn write_all(&mut self, buf: &[u8]) {
        self.write(buf);
    }
    fn flush_internal(rx: &mpsc::Receiver<os_pipe::PipeWriter>, pipes: &mut Vec<Receiver>, data: &[u8]) {
        if data.len() == 0 {
            return;
        }
        loop {
            let mut completed = 0;
            pipes.extend(rx.try_iter().map(|pipe| {
                Receiver { pipe, at: 0 }
            }));
            pipes.retain_mut(|r| {
                if r.at >= data.len() {
                    completed += 1;
                    return true;
                }
                r.pipe.write(&data[r.at..]).inspect(|len| {
                    r.at += len;
                }).is_ok()
            });
            if completed == data.len() {
                break;
            }
        }
    }
    pub fn flush(&mut self) {
        Self::flush_internal(&self.rx, &mut self.pipes, &self.data);
    }
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    pub fn release_data_and_flush(mut self, func: impl FnOnce(Arc<[u8]>)) {
        let data: Arc<[u8]> = std::mem::replace(&mut self.data, Vec::new()).into();
        func(data.clone());
        Self::flush_internal(&self.rx, &mut self.pipes, &data);
    }
    pub fn into_data(mut self) -> Vec<u8> {
        self.flush();
        std::mem::replace(&mut self.data, Vec::new())
    }
}

impl Write for PartialCacheWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(self.write(buf))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(self.flush())
    }
}

impl Drop for PartialCacheWriter {
    fn drop(&mut self) {
        self.flush();
    }
}

