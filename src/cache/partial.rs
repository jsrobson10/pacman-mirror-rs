
use std::{io::Write, sync::Arc};
use owner::PartialCacheOwner;
pub use reader::PartialCacheReader;

pub mod owner;
pub mod reader;

#[cfg(test)]
mod tests;

pub struct PartialCache<T> {
	src: Arc<PartialCacheOwner<T>>,
}

impl<T> PartialCache<T> {
	pub fn new() -> PartialCache<T> {
		Self { src: PartialCacheOwner::new(Vec::new(), None) }
	}
	pub fn with_capacity(len: usize) -> PartialCache<T> {
		Self { src: PartialCacheOwner::new(Vec::with_capacity(len), Some(len)) }
	}
	pub fn reader(&self) -> PartialCacheReader<T> {
		PartialCacheReader::new(self)
	}
	fn with_vec_mut(&self, func: impl FnOnce(&mut Vec<T>)) {
		func(&mut self.src.data.lock().unwrap().items);
		self.src.cvar.notify_all();
	}
	pub fn extend(&self, it: impl IntoIterator<Item=T>) {
		self.with_vec_mut(|items| items.extend(it));
	}
	pub fn push(&self, item: T) {
		self.with_vec_mut(|items| items.push(item));
	}
	pub fn get_buffer_size(&self) -> usize {
		self.src.get_buffer_size()
	}
}
impl Write for PartialCache<u8> {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.extend(buf.iter().copied());
		Ok(buf.len())
	}
	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}
impl<T> Drop for PartialCache<T> {
	fn drop(&mut self) {
		if let Ok(mut data) = self.src.data.lock() {
			data.done = true;
			self.src.cvar.notify_all();
		}
	}
}
impl<T> Into<Arc<[T]>> for PartialCache<T> where T: Clone {
	fn into(self) -> Arc<[T]> {
		self.src.data.lock().unwrap().items.clone().into()
	}
}
