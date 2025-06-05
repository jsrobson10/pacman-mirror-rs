use std::{collections::HashMap, sync::{Arc, RwLock, RwLockReadGuard}, time::Instant};
use lazy_static::lazy_static;
use repo::Repo;

use crate::config::CONFIG;


pub mod desc;
pub mod mirror;
pub mod package;
pub mod repo;

pub struct Database {
	pub repos: HashMap<Arc<str>, RwLock<Repo>>,
}

impl Database {
	fn new() -> Self {
		let mut repos = HashMap::new();
		for name in CONFIG.repos.iter() {
			repos.insert(name.clone(), RwLock::new(Repo::empty()));
		}
		Self { repos }
	}
	fn force_refresh(&self, name: &str) -> Option<RwLockReadGuard<Repo>> {
		let repo = self.repos.get(name)?;
		let mut wlock = repo.write().unwrap();
		*wlock = Repo::new(&name);
		drop(wlock);
		Some(repo.read().unwrap())
	}
	pub fn get_or_refresh(&self, name: &str) -> Option<RwLockReadGuard<Repo>> {
		let rlock = self.repos.get(name)?.read().unwrap();

		if rlock.created.elapsed() >= CONFIG.timeout {
			drop(rlock);
			self.force_refresh(name)
		} else {
			Some(rlock)
		}
	}
}

lazy_static! {
	pub static ref DB: Database = Database::new();
}

