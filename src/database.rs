use std::{collections::HashMap, sync::Arc};
use lazy_static::lazy_static;
use repo::holder::RepoHolder;

use crate::config::CONFIG;


pub mod desc;
pub mod mirror;
pub mod package;
pub mod repo;

pub struct Database {
    pub repos: HashMap<Arc<str>, RepoHolder>,
}

impl Database {
    fn new() -> Self {
        let mut repos = HashMap::new();
        for name in CONFIG.repos.iter() {
            repos.insert(name.clone(), RepoHolder::new(name.clone()));
        }
        Self { repos }
    }
}

lazy_static! {
    pub static ref DB: Database = Database::new();
}

