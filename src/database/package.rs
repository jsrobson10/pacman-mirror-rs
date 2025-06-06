use crate::cache::Cache;

use super::{desc::Desc, mirror::Mirror};

pub struct Package {
	pub desc: Desc,
	pub cache: Cache,
	pub mirrors: Vec<Mirror>,
	pub files: Option<String>,
}

impl Package {
	pub fn new(desc: Desc) -> Self {
		Self {
			desc,
			cache: Cache::new(),
			mirrors: Vec::new(),
			files: None,
		}
	}
}

