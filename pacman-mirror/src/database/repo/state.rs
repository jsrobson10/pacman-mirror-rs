use std::{collections::HashMap, sync::Arc, time::SystemTime};

use crate::{database::package::Package, Config};


pub struct State {
    pub packages: HashMap<Arc<str>, Package>,
    pub packages_by_filename: HashMap<Arc<str>, Arc<str>>,
    pub last_updated: SystemTime,
    pub ty: FetchType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FetchType {
    Db, Files,
}

impl Default for State {
    fn default() -> Self {
        Self {
            packages: HashMap::new(),
            packages_by_filename: HashMap::new(),
            last_updated: SystemTime::UNIX_EPOCH,
            ty: FetchType::Db,
        }
    }
}

impl State {
    pub fn should_refresh(&self, config: &Config, ty: FetchType) -> bool {
        ty > self.ty || self.last_updated.elapsed().is_ok_and(|v| v > config.timeout)
    }
}

