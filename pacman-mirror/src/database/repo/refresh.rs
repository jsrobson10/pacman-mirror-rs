use std::{sync::{atomic::{self, AtomicBool}, mpsc}, time::SystemTime};

use iter_iterator::IterIterator;
use itertools::Itertools;
use log::{debug, error, info};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::database::{package::Package, Repo};


struct AtomicStoreGuard<'a>(&'a AtomicBool);

impl<'a> Drop for AtomicStoreGuard<'a> {
    fn drop(&mut self) {
        self.0.store(false, atomic::Ordering::Relaxed);
    }
}

impl Repo {
    pub fn should_refresh(&self, files: bool) -> bool {
        self.is_updating.load(atomic::Ordering::Relaxed) || self.state.read().unwrap().should_refresh(&self.config, files)
    }
    pub fn try_refresh(&self, signal: Option<mpsc::Sender<()>>, files: bool) {
        if self.is_updating.swap(true, atomic::Ordering::Relaxed) {
            return;
        }

        let _guard = AtomicStoreGuard(&self.is_updating);
        let repo_name = &self.name;
        debug!("Refreshing {repo_name}");

        let mut buf_writers = self.mirrors.iter()
            .map(|v| (v, v.prepare_for_update()))
            .collect_vec();
        drop(signal);

        buf_writers.par_iter_mut().for_each(|(mirror, writer)| {
            if let Err(err) = mirror.update(writer, false) {
                error!("mirror {}: {err:?}", mirror.repo_url);
            }
        });

        let iter = IterIterator::new(buf_writers.into_iter()
            .map(|(mirror, writer)| (writer.source().clone().read(), mirror))
            .collect());

        let mut state = self.state.write().unwrap();
        state.last_updated = SystemTime::now();
        state.files = files;

        let mut added = 0;
        let mut removed = 0;

        for pkg in state.packages.values_mut() {
            pkg.mirrors.clear();
        }
        for (desc, mirror) in iter {
            let pkg = state.packages.entry(desc.name.clone()).or_insert_with(|| {
                added += 1;
                Package::new(desc.clone())
            });
            if vercmp::alpm_pkg_ver_cmp(&desc.version, &pkg.desc.version) == std::cmp::Ordering::Greater {
                *pkg = Package::new(desc.clone());
            }
            pkg.mirrors.push(mirror.repo_url.clone());
        }
        state.packages.retain(|_, pkg| {
            let r = pkg.mirrors.len() > 0;
            if !r { removed += 1 };
            r
        });

        state.packages_by_filename = state.packages.values()
            .map(|pkg| (pkg.desc.filename.clone(), pkg.desc.name.clone()))
            .collect();

        info!("Refreshed {repo_name}: {added} added {removed} removed");
    }
}

