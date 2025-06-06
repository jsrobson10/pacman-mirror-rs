use std::{collections::HashMap, sync::{Arc, RwLock}};

pub use data_source::DataSource;
use owning_ref::ArcRef;

use super::mirror::Mirror;

pub mod data_source;

pub struct Package {
	pub name: Arc<str>,
	pub mirrors: Vec<Mirror>,
	pub source: RwLock<DataSource>,
	pub desc: Option<HashMap<ArcRef<str>, ArcRef<str>>>,
	pub files: Option<String>,
}

impl Package {
	pub fn new(name: Arc<str>) -> Self {
		Self {
			name,
			mirrors: Vec::new(),
			source: RwLock::new(DataSource::None),
			desc: None,
			files: None,
		}
	}
}

