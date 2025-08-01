use std::sync::{Arc, RwLock};

use replay_buffer::{ReplayBuffer, ReplayBufferWriter};

use crate::{database::{desc::Desc, mirror::Mirror}, Config};

mod update;


pub struct State {
    pub packages: Arc<ReplayBuffer<Arc<Desc>>>,
}

pub struct MirrorData {
    pub repo_name: Arc<str>,
    pub repo_url: Arc<str>,
    pub state: RwLock<State>,
}

impl MirrorData {
    pub fn new(config: &Config, mirror: &Mirror, repo_name: Arc<str>) -> Self {
        let repo_url: Arc<str> = mirror.get(config, &repo_name).into();
        Self {
            repo_name,
            repo_url,
            state: RwLock::new(State {
                packages: ReplayBufferWriter::new().source().clone(),
            }),
        }
    }
}

