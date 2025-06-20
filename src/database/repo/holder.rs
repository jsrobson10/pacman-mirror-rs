use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::config::CONFIG;

use super::Repo;


pub struct RepoHolder {
    repo: RwLock<Repo>,
    pub name: Arc<str>,
}

impl RepoHolder {
    pub fn new(name: Arc<str>) -> Self {
        Self { name, repo: RwLock::new(Repo::empty()) }
    }
    fn force_refresh(&self) -> RwLockReadGuard<Repo> {
        let mut wlock = self.repo.write().unwrap();
        wlock.refresh(&self.name);
        drop(wlock);
        self.repo.read().unwrap()
    }
    pub fn get_without_refresh(&self) -> RwLockReadGuard<Repo> {
        self.repo.read().unwrap()
    }
    pub fn get_or_refresh(&self) -> RwLockReadGuard<Repo> {
        let rlock = self.repo.read().unwrap();
        if rlock.last_updated.elapsed().map_or(false, |v| v >= CONFIG.timeout) {
            drop(rlock);
            self.force_refresh()
        } else {
            rlock
        }
    }
}

