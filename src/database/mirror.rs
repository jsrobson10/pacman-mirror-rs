use std::sync::Arc;

use serde::{Deserialize, Serialize};
use crate::config::CONFIG;


#[derive(Debug,Serialize,Deserialize,Eq,Hash,PartialEq,Clone)]
pub struct Mirror(Arc<str>);

impl Mirror {
    pub fn new(path: String) -> Self {
        Self(path.into())
    }
    pub fn get(&self, repo: &str) -> String {
        self.0
            .replace("$repo", repo)
            .replace("$name", &CONFIG.name)
            .replace("$arch", &CONFIG.arch)
    }
}

