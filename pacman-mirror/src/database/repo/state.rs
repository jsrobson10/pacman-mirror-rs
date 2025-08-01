use std::{collections::HashMap, sync::Arc, time::SystemTime};

use crate::{database::package::Package, Config};


pub struct State {
    pub packages: HashMap<Arc<str>, Package>,
    pub packages_by_filename: HashMap<Arc<str>, Arc<str>>,
    pub last_updated: SystemTime,
    pub files: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            packages: HashMap::new(),
            packages_by_filename: HashMap::new(),
            last_updated: SystemTime::UNIX_EPOCH,
            files: false,
        }
    }
}

impl State {
    pub fn should_refresh(&self, config: &Config, files: bool) -> bool {
        files && !self.files || self.last_updated.elapsed().is_ok_and(|v| v > config.timeout)
    }
}

