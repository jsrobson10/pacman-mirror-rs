use std::{collections::HashMap, sync::Arc};
use crate::Config;

pub use repo::Repo;

pub mod desc;
pub mod mirror;
pub mod package;
pub mod repo;
pub mod mirror_data;

pub struct Database {
    pub repos: HashMap<Arc<str>, Arc<Repo>>,
    pub config: Arc<Config>,
}

impl Database {
    pub fn new(config: Arc<Config>) -> Self {
        let mut repos = HashMap::new();
        for name in config.repos.iter().cloned() {
            repos.insert(name.clone(), Arc::new(Repo::empty(config.clone(), name)));
        }
        Self { repos, config }
    }
}

