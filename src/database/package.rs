use std::{collections::{HashMap, HashSet}, sync::Arc};

use super::mirror::Mirror;

#[derive(Debug)]
pub struct Package {
	pub name: String,
	pub mirrors: HashSet<Arc<Mirror>>,
	pub desc: HashMap<String, String>,
	pub files: String,
}

impl Package {
	pub fn new(name: String) -> Self {
		Self {
			name,
			mirrors: HashSet::new(),
			desc: HashMap::new(),
			files: "".into(),
		}
	}
}

