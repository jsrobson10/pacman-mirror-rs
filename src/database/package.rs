use std::{collections::{HashMap, HashSet}, sync::{Arc, RwLock}};

pub use data_source::DataSource;

use super::mirror::Mirror;

pub mod data_source;

pub struct Package {
	pub name: Arc<str>,
	pub mirrors: HashSet<Mirror>,
	pub source: RwLock<DataSource>,
	pub desc: Option<String>,
	pub files: Option<String>,
}

impl Package {
	pub fn new(name: Arc<str>) -> Self {
		Self {
			name,
			mirrors: HashSet::new(),
			source: RwLock::new(DataSource::None),
			desc: None,
			files: None,
		}
	}
}

