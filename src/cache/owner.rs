use std::sync::{Arc, Condvar, Mutex};


pub(super) struct CacheData<T> {
	pub items: Vec<T>,
	pub done: bool,
}

pub(super) struct CacheOwner<T> {
	pub data: Mutex<CacheData<T>>,
	pub size_hint: Option<usize>,
	pub cvar: Condvar,
}

impl<T> CacheOwner<T> {
	pub fn new(items: Vec<T>, size_hint: Option<usize>) -> Arc<Self> {
		Arc::new(Self {
			data: Mutex::new(CacheData { items, done: false }),
			cvar: Condvar::new(),
			size_hint,
		})
	}
	pub fn wait_for_data_lock(&self, at: usize) -> std::sync::MutexGuard<CacheData<T>> {
		let mut data = self.data.lock().unwrap();
		while !data.done && at >= data.items.len() {
			data = self.cvar.wait(data).unwrap();
		}
		data
	}
}

