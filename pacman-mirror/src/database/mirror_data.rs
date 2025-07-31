use std::sync::{Arc, RwLock};

use replay_buffer::ReplayBuffer;

use crate::database::desc::Desc;

mod update;
mod get_all;


pub struct State {
    packages: Arc<ReplayBuffer<Arc<Desc>>>,
}

pub struct MirrorData {
    pub repo_url: Arc<str>,
    state: RwLock<State>,
}

impl MirrorData {
    pub fn new(repo_url: Arc<str>) -> Self {
        Self {
            repo_url,
            state: RwLock::new(State {
                packages: ReplayBuffer::empty(),
            }),
        }
    }
}

