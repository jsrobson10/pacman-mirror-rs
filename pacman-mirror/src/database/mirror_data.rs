use std::{path::Path, sync::{Arc, RwLock}};

use replay_buffer::ReplayBuffer;

use crate::{database::{desc::Desc, mirror::Mirror}, Config};

mod update;
mod get_all;


pub struct State {
    packages: Arc<ReplayBuffer<Arc<Desc>>>,
}

pub struct MirrorData {
    pub repo_url: Arc<str>,
    db_url: Box<str>,
    state: RwLock<State>,
}

impl MirrorData {
    pub fn new(config: &Config, mirror: &Mirror, name: &str) -> Self {
        let repo_url: Arc<str> = mirror.get(config, name).into();
        let db_url = Path::new(repo_url.as_ref())
            .join(format!("{name}.db"))
            .to_string_lossy()
            .into();
        Self {
            repo_url,
            db_url,
            state: RwLock::new(State {
                packages: ReplayBuffer::empty(),
            }),
        }
    }
}

