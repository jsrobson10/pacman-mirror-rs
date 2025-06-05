use std::{collections::{HashMap, HashSet}, sync::Arc};

use super::mirror::Mirror;

#[derive(Debug)]
pub struct Package {
	pub name: Arc<str>,
	pub mirrors: HashSet<Mirror>,
	pub desc: HashMap<String, String>,
	pub files: String,
}

impl Package {
	pub fn new(name: Arc<str>) -> Self {
		Self {
			name,
			mirrors: HashSet::new(),
			desc: HashMap::new(),
			files: "".into(),
		}
	}
}

