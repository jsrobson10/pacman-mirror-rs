use std::{collections::HashMap, sync::RwLock};
use lazy_static::lazy_static;
use repo::Repo;

use crate::config::CONFIG;


pub mod desc;
pub mod mirror;
pub mod package;
pub mod repo;

pub struct Database {
	pub repos: HashMap<String, RwLock<Repo>>,
}

impl Database {
	fn new() -> Self {
		let mut repos = HashMap::new();

		for name in CONFIG.repos.iter() {
		}

		Self { repos }
	}
}

lazy_static! {
	pub static ref DB: Database = Database::new();
}

