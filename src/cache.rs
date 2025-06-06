use std::sync::Mutex;

pub use data_source::DataSource;
pub use partial::{PartialCache, PartialCacheReader};

pub mod partial;
pub mod data_source;

pub struct Cache {
	src: Mutex<DataSource>,
}

impl Cache {
	pub fn new() -> Self {
		Self { src: DataSource::Empty.into() }
	}
	pub fn set(&self, source: DataSource) {
		*self.src.lock().unwrap() = source;
	}
	pub fn get(&self) -> DataSource {
		self.src.lock().unwrap().clone()
	}
}

