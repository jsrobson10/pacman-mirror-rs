use serde::{Deserialize, Serialize};

use crate::Config;


#[derive(Debug,Serialize,Deserialize,Eq,Hash,PartialEq,Clone)]
pub struct Mirror(Box<str>);

impl Mirror {
    pub fn new(path: Box<str>) -> Self {
        Self(path)
    }
    pub fn get(&self, config: &Config, repo: &str) -> String {
        self.0
            .replace("$repo", repo)
            .replace("$name", &config.name)
            .replace("$arch", &config.arch)
    }
}

