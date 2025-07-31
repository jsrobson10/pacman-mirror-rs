use std::{cmp::Ordering, collections::HashMap, sync::{mpsc, Arc, Mutex, RwLock}, time::SystemTime};
use iter_iterator::IterIterator;
use itertools::Itertools;
use log::{debug, error, info};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use crate::{database::mirror_data::MirrorData, Config};

use super::package::Package;


pub struct Repo {
    pub name: Arc<str>,
    pub config: Arc<Config>,
    pub mirrors: Vec<MirrorData>,
    pub packages: RwLock<HashMap<Arc<str>, Package>>,
    package_filename_lookup: RwLock<HashMap<Arc<str>, Arc<str>>>,
    last_updated: Mutex<SystemTime>,
}

impl Repo {
    pub fn empty(config: Arc<Config>, name: Arc<str>) -> Repo {
        let mirrors = Vec::from_iter(config.mirrors.iter()
            .map(|mirror| MirrorData::new(&config, mirror, &name)));
        Self {
            name,
            config,
            mirrors,
            packages: RwLock::new(HashMap::new()),
            package_filename_lookup: RwLock::new(HashMap::new()),
            last_updated: Mutex::new(SystemTime::UNIX_EPOCH),
        }
    }
    pub fn should_refresh(&self) -> bool {
        let last_updated = self.last_updated.lock().unwrap();
        match last_updated.elapsed().map(|v| v > self.config.timeout) {
            Ok(true) => true,
            _ => false,
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
    pub fn get_name_from_filename(&self, filename: &str) -> Option<Arc<str>> {
        self.package_filename_lookup.read().unwrap().get(filename).cloned()
    }
    pub fn refresh_if_ready(&self, signal: Option<mpsc::Sender<()>>) {
        if !self.will_refresh() {
            return;
        }

        let repo_name = &self.name;
        debug!("Refreshing {repo_name}");

        let mut buf_writers = self.mirrors.iter()
            .map(|v| (v, v.prepare_for_update()))
            .collect_vec();
        drop(signal);

        buf_writers.par_iter_mut().for_each(|(mirror, writer)| {
            if let Err(err) = mirror.update(writer) {
                error!("mirror {}: {err:?}", mirror.repo_url);
            }
        });

        let iter = IterIterator::new(buf_writers.into_iter()
            .map(|(mirror, writer)| (writer.source().clone().read(), mirror))
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

        *self.package_filename_lookup.write().unwrap() = packages.values()
            .map(|pkg| (pkg.desc.filename.clone(), pkg.desc.name.clone()))
            .collect();

        info!("Refreshed {repo_name}: {added} added {removed} removed");
    }
}

