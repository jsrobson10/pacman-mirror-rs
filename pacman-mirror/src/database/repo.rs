use std::sync::{atomic::AtomicBool, Arc, RwLock};
use crate::{database::mirror_data::MirrorData, Config};

pub use state::State;

pub mod state;
mod refresh;
mod get_all;


pub struct Repo {
    pub name: Arc<str>,
    pub config: Arc<Config>,
    pub mirrors: Vec<MirrorData>,
    pub state: RwLock<State>,
    is_updating: AtomicBool,
}

impl Repo {
    pub fn empty(config: Arc<Config>, name: Arc<str>) -> Repo {
        let mirrors = Vec::from_iter(config.mirrors.iter()
            .map(|mirror| MirrorData::new(&config, mirror, name.clone())));
        Self {
            name,
            config,
            mirrors,
            state: RwLock::new(State::default()),
            is_updating: AtomicBool::new(false),
        }
    }
}

