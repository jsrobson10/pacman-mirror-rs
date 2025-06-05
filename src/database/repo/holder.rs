use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::config::CONFIG;

use super::Repo;


pub struct RepoHolder {
	repo: RwLock<Repo>,
	name: Arc<str>,
}

impl RepoHolder {
	pub fn new(name: Arc<str>) -> Self {
		Self { name, repo: RwLock::new(Repo::empty()) }
	}
	fn force_refresh(&self) -> RwLockReadGuard<Repo> {
		let mut wlock = self.repo.write().unwrap();
		*wlock = Repo::new(&self.name);
		drop(wlock);
		self.repo.read().unwrap()
	}
	pub fn get_or_refresh(&self) -> RwLockReadGuard<Repo> {
		let rlock = self.repo.read().unwrap();

		if rlock.created.elapsed() >= CONFIG.timeout {
			drop(rlock);
			self.force_refresh()
		} else {
			rlock
		}
	}
}

