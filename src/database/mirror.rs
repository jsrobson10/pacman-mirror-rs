use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::Config;


#[derive(Debug,Serialize,Deserialize,Eq,Hash,PartialEq,Clone)]
pub struct Mirror(Arc<str>);

impl Mirror {
    pub fn new(path: String) -> Self {
        Self(path.into())
    }
    pub fn get(&self, config: &Config, repo: &str) -> String {
        self.0
            .replace("$repo", repo)
            .replace("$name", &config.name)
            .replace("$arch", &config.arch)
    }
}

