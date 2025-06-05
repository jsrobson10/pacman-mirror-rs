use std::{collections::{HashMap, HashSet}, sync::Arc};

use super::mirror::Mirror;

#[derive(Debug)]
pub struct Package {
	pub name: Arc<str>,
	pub mirrors: HashSet<Mirror>,
	pub desc: Option<String>,
	pub files: Option<String>,
}

impl Package {
	pub fn new(name: Arc<str>) -> Self {
		Self {
			name,
			mirrors: HashSet::new(),
			desc: None,
			files: None,
		}
	}
}

