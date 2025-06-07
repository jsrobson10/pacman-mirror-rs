use owning_ref::ArcRef;

use crate::cache::Cache;

use super::{desc::Desc, mirror::Mirror};

pub struct Package {
	pub desc: Desc,
	pub cache: Cache,
	pub mirrors: Vec<Mirror>,
	pub files: Option<String>,
}

pub struct PackageRef {
	pub filename: ArcRef<str>,
	pub version: ArcRef<str>,
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

impl PackageRef {
	pub fn new(desc: &Desc) -> Self {
		Self {
			filename: desc.filename.clone(),
			version: desc.version.clone(),
		}
	}
}

