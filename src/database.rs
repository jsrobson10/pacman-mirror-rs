use std::{collections::HashMap, sync::Arc};
use repo::holder::RepoHolder;
use crate::Config;


pub mod desc;
pub mod mirror;
pub mod package;
pub mod repo;

pub struct Database {
    pub repos: HashMap<Arc<str>, Arc<RepoHolder>>,
    pub config: Arc<Config>,
}

impl Database {
    pub fn new(config: Arc<Config>) -> Self {
        let mut repos = HashMap::new();
        for name in config.repos.iter().cloned() {
            repos.insert(name.clone(), Arc::new(RepoHolder::new(config.clone(), name)));
        }
        Self { repos, config }
    }
}

