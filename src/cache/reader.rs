use std::sync::Arc;
use super::{Cache, CacheOwner};


pub struct CacheReader<T> {
	src: Arc<CacheOwner<T>>,
	at: usize,
}

impl<T> CacheReader<T> {
	pub(super) fn new(reader: &Cache<T>) -> Self {
		Self { src: reader.src.clone(), at: 0 }
	}
	pub fn reader(&self) -> Self {
		Self { src: self.src.clone(), at: 0 }
	}
	pub fn read_with(&mut self, func: impl FnOnce(&[T]) -> usize) -> usize {
		let data = self.src.wait_for_data_lock(self.at);
		let len = func(&data.items[self.at..]);
		self.at += len;
		len
	}
	pub fn read_for_each(&mut self, func: impl FnMut(&T)) -> usize {
		self.read_with(|src| src.iter().map(func).count())
	}
	pub fn read_into_slice(&mut self, dst: &mut [T]) -> usize where T: Clone {
		self.read_with(|src| src.iter().zip(dst.iter_mut()).map(|(src, dst)| *dst = src.clone()).count())
	}
	pub fn read_into_vec(&mut self, dst: &mut Vec<T>) -> usize where T: Clone {
		self.read_with(|src| { dst.extend_from_slice(src); src.len() })
	}
	pub fn read_vec(&mut self) -> Option<Vec<T>> where T: Clone {
		let mut dst = Vec::new();
		if self.read_into_vec(&mut dst) == 0 {
			return None;
		}
		Some(dst)
	}
	pub fn get_size_hint(&self) -> Option<usize> {
		self.src.size_hint
	}
}

impl std::io::Read for CacheReader<u8> {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		Ok(self.read_into_slice(buf))
	}
}

