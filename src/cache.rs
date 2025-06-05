use std::sync::Arc;
use owner::CacheOwner;
pub use reader::CacheReader;

pub mod owner;
pub mod reader;

#[cfg(test)]
mod tests;

pub struct Cache<T> {
	src: Arc<CacheOwner<T>>,
}

impl<T> Cache<T> {
	pub fn new() -> Cache<T> {
		Self { src: CacheOwner::new(Vec::new(), None) }
	}
	pub fn with_capacity(len: usize) -> Cache<T> {
		Self { src: CacheOwner::new(Vec::with_capacity(len), Some(len)) }
	}
	pub fn reader(&self) -> CacheReader<T> {
		CacheReader::new(self)
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
}

impl<T> Drop for Cache<T> {
	fn drop(&mut self) {
		if let Ok(mut data) = self.src.data.lock() {
			data.done = true;
			self.src.cvar.notify_all();
		}
	}
}

