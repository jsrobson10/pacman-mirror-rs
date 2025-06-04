use serde::{Deserialize, Serialize};
use crate::config::CONFIG;


#[derive(Debug,Serialize,Deserialize,Eq,Hash,PartialEq)]
pub struct Mirror(String);

impl Mirror {
	pub fn new(path: String) -> Self {
		Self(path)
	}
	pub fn get(&self, repo: &str) -> String {
		self.0
			.replace("$repo", repo)
			.replace("$name", &CONFIG.name)
			.replace("$arch", &CONFIG.arch)
	}
}

