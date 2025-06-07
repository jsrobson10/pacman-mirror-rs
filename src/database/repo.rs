use std::{cmp::Ordering, collections::{hash_map::Entry, HashMap}, io::Read, path::PathBuf, sync::Mutex, time::SystemTime};
use flate2::read::GzDecoder;
use itertools::Itertools;
use log::{debug, error, info};
use owning_ref::ArcRef;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use crate::{config::CONFIG, vercmp};

use super::{desc::Desc, mirror::Mirror, package::{Package, PackageRef}};

pub mod holder;

pub struct Repo {
	pub last_updated: SystemTime,
	pub packages: HashMap<ArcRef<str>, Package>,
	pub packages_by_name: HashMap<ArcRef<str>, PackageRef>,
}

fn get_from_repo(repo_name: &str, mirror: &Mirror) -> anyhow::Result<Vec<Package>> {
	let url = format!("{}.db", PathBuf::from(mirror.get(&repo_name)).join(repo_name).to_string_lossy());
	let res = minreq::get(&url).send_lazy()?;
	let mut data = Vec::new();

	if res.status_code != 200 {
		anyhow::bail!("Request {url} failed with code {}: {}", res.status_code, res.reason_phrase);
	}

	let mut archive = tar::Archive::new(GzDecoder::new(res));
	for entry in archive.entries()? {
		let mut entry = entry?;
		let path = entry.path()?.into_owned();
		let [_, ty] = match path.iter().flat_map(|v| v.to_str()).collect_array() {
			None => continue,
			Some(v) => v,
		};
		if ty != "desc" {
			continue;
		}
		let desc = {
			let mut dst = String::new();
			entry.read_to_string(&mut dst)?;
			Desc::parse(dst.into())?
		};
		data.push(Package::new(desc));
	}
	Ok(data)
}

impl Repo {
	pub fn empty() -> Repo {
		Self {
			last_updated: SystemTime::UNIX_EPOCH,
			packages: HashMap::new(),
			packages_by_name: HashMap::new(),
		}
	}
	pub fn refresh(&mut self, repo_name: &str) {
		self.packages_by_name.clear();
		self.packages.values_mut().for_each(|pkg| pkg.mirrors.clear());

		debug!("Refreshing database");

		let size_start = self.packages.len();
		let packages = Mutex::new((&mut self.packages, &mut self.packages_by_name));

		CONFIG.mirrors.par_iter().for_each(|mirror| {
			let l_packages = match get_from_repo(repo_name, mirror) {
				Ok(data) => data,
				Err(err) => {
					error!("{err}");
					return;
				}
			};
			let mut lock = packages.lock().unwrap();
			let (ref mut packages, ref mut packages_by_name) = *lock;

			for mut package in l_packages {
				match packages_by_name.entry(package.desc.name.clone()) {
					Entry::Vacant(dst) => {
						dst.insert(PackageRef::new(&package.desc));
					}
					Entry::Occupied(mut dst) => {
						let dst = dst.get_mut();
						if vercmp::alpm_pkg_ver_cmp(package.desc.version.as_ref(), dst.version.as_ref()) == Ordering::Greater {
							*dst = PackageRef::new(&package.desc);
						}
					}
				}
				match packages.entry(package.desc.filename.clone()) {
					Entry::Vacant(dst) => {
						package.mirrors.push(mirror.clone());
						dst.insert(package);
					}
					Entry::Occupied(mut dst) => {
						let dst = dst.get_mut();
						if !dst.mirrors.contains(mirror) {
							dst.mirrors.push(mirror.clone());
						}
					}
				}
			}
		});

		let size_end = self.packages.len();
		self.packages.retain(|_, pkg| pkg.mirrors.len() > 0);
		self.last_updated = SystemTime::now();

		info!("Refreshed {repo_name}: {} added {} removed", size_end - size_start, size_end - self.packages.len());
	}
}

