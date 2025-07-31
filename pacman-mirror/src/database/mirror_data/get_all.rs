use std::{cmp::Ordering, collections::HashMap, sync::Arc};

use iter_iterator::IterIterator;
use log::debug;

use crate::{database::{desc::Desc, repo::Repo}};


struct PackageWithCount {
    desc: Arc<Desc>,
    count: usize,
}

impl PackageWithCount {
    fn new(desc: Arc<Desc>) -> Self {
        Self { desc, count: 0 }
    }
}

impl Repo {
    pub fn get_from_mirrors<T>(&self, mut callback: impl FnMut(Arc<Desc>) -> Result<(), T>) -> Result<(), T> {
        
        let mirror_count = self.config.mirrors.len();
        let mut packages = HashMap::<Arc<str>, PackageWithCount>::new();
        let iter = IterIterator::new(self.mirrors.iter()
            .map(|v| (v.state.read().unwrap().packages.read(), ()))
            .collect());

        for (desc, _) in iter {
            let pkg = packages.entry(desc.name.clone())
                .or_insert_with(|| PackageWithCount::new(desc.clone()));
            if vercmp::alpm_pkg_ver_cmp(&desc.version, &pkg.desc.version) == Ordering::Greater {
                pkg.desc = desc;
            }
            pkg.count += 1;
            if pkg.count == mirror_count {
                callback(pkg.desc.clone())?;
            }
        }
        for pkg in packages.values() {
            if pkg.count < mirror_count {
                callback(pkg.desc.clone())?;
            }
        }
        Ok(())
    }
}

