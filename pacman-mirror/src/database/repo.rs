use std::{cmp::Ordering, collections::HashMap, sync::{Arc, Mutex, RwLock}, time::SystemTime};
use iter_iterator::IterIterator;
use log::{debug, error, info};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use crate::{database::mirror_data::MirrorData, Config};

use super::package::Package;


pub struct Repo {
    pub name: Arc<str>,
    pub config: Arc<Config>,
    pub mirrors: HashMap<Arc<str>, MirrorData>,
    pub packages: RwLock<HashMap<Arc<str>, Package>>,
    pub last_updated: Mutex<SystemTime>,
}

impl Repo {
    pub fn empty(config: Arc<Config>, name: Arc<str>) -> Repo {
        Self {
            name,
            config,
            mirrors: HashMap::new(),
            packages: RwLock::new(HashMap::new()),
            last_updated: Mutex::new(SystemTime::UNIX_EPOCH),
        }
    }
    fn will_refresh(&self) -> bool {
        let mut last_updated = self.last_updated.lock().unwrap();
        match last_updated.elapsed().map(|v| v > self.config.timeout) {
            Ok(true) => {
                *last_updated = SystemTime::now();
                true
            }
            _ => false,
        }
    }
    pub fn refresh_if_ready(&self) {
        if !self.will_refresh() {
            return;
        }

        let repo_name = &self.name;
        debug!("Refreshing ${repo_name}");

        let iter = IterIterator::new(self.mirrors.par_iter()
            .flat_map(|(_, mirror)| mirror.update()
                .inspect_err(|err| error!("mirror {}: {err:?}", mirror.repo_url))
                .map(|iter| (iter, mirror))
                .ok())
            .collect());
        let mut packages = self.packages.write().unwrap();
        let mut added = 0;
        let mut removed = 0;

        for pkg in packages.values_mut() {
            pkg.mirrors.clear();
        }
        for (desc, mirror) in iter {
            let pkg = packages.entry(desc.name.clone()).or_insert_with(|| {
                added += 1;
                Package::new(desc.clone())
            });
            if vercmp::alpm_pkg_ver_cmp(&desc.version, &pkg.desc.version) == Ordering::Greater {
                *pkg = Package::new(desc.clone());
            }
            pkg.mirrors.push(mirror.repo_url.clone());
        }
        packages.retain(|_, pkg| {
            let r = pkg.mirrors.len() > 0;
            if !r { removed += 1 };
            r
        });

        info!("Refreshed {repo_name}: {added} added {removed} removed");
    }
}

