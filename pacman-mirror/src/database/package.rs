use std::sync::Arc;

use crate::cache::Cache;

use super::desc::Desc;

pub struct Package {
    pub desc: Arc<Desc>,
    pub cache: Cache,
    pub mirrors: Vec<Arc<str>>,
    pub files: Option<String>,
}

impl Package {
    pub fn new(desc: Arc<Desc>) -> Self {
        Self {
            desc,
            cache: Cache::new(),
            mirrors: Vec::new(),
            files: None,
        }
    }
}

