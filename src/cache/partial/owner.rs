use std::sync::{Arc, Condvar, Mutex};


pub(super) struct PartialCacheData<T> {
	pub items: Vec<T>,
	pub done: bool,
}

pub(super) struct PartialCacheOwner<T> {
	pub data: Mutex<PartialCacheData<T>>,
	pub size_hint: Option<usize>,
	pub cvar: Condvar,
}

impl<T> PartialCacheOwner<T> {
	pub fn new(items: Vec<T>, size_hint: Option<usize>) -> Arc<Self> {
		Arc::new(Self {
			data: Mutex::new(PartialCacheData { items, done: false }),
			cvar: Condvar::new(),
			size_hint,
		})
	}
	pub fn wait_for_data_lock(&self, at: usize) -> std::sync::MutexGuard<PartialCacheData<T>> {
		let mut data = self.data.lock().unwrap();
		while !data.done && at >= data.items.len() {
			data = self.cvar.wait(data).unwrap();
		}
		data
	}
	pub fn get_buffer_size(&self) -> usize {
		self.data.lock().unwrap().items.len()
	}
}

