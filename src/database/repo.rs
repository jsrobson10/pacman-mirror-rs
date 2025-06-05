use std::{collections::HashMap, io::Read, path::PathBuf, sync::{Arc, Mutex}, time::Instant};
use flate2::read::GzDecoder;
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use crate::config::{self, CONFIG};

use super::package::Package;

pub mod holder;

pub struct Repo {
	pub created: Instant,
	pub packages: HashMap<Arc<str>, Package>,
}

impl Repo {
	pub fn empty() -> Repo {
		Self {
			created: Instant::now() - CONFIG.timeout,
			packages: HashMap::new(),
		}
	}
	pub fn new(name: &str) -> Repo {
		let packages: Mutex<HashMap<Arc<str>, Package>> = Mutex::new(HashMap::new());
		
		config::CONFIG.mirrors.par_iter().map(|mirror| -> anyhow::Result<()> {
			let url = [mirror.get(&name), format!("{name}.db")].into_iter().collect::<PathBuf>().to_string_lossy().into_owned();
			let res = minreq::get(&url).send_lazy()?;

			if res.status_code != 200 {
				anyhow::bail!("Request {url} failed with code {}: {}", res.status_code, res.reason_phrase);
			}

			let mut archive = tar::Archive::new(GzDecoder::new(res));
			for entry in archive.entries()? {
				let mut entry = entry?;
				let path = entry.path()?.into_owned();
				let [pkg_name, ty] = match path.iter().flat_map(|v| v.to_str()).collect_array() {
					None => continue,
					Some(v) => v,
				};
				
				let pkg_name: Arc<str> = pkg_name.into();
				let mut packages = packages.lock().unwrap();
				let dst = packages.entry(pkg_name.clone()).or_insert_with(|| Package::new(pkg_name.clone()));
				dst.mirrors.insert(mirror.clone());

				match ty {
					"desc" => {
						if dst.desc.is_none() {
							let mut str = String::new();
							entry.read_to_string(&mut str)?;
							dst.desc = Some(str);
						}
					}
					"files" => {
						if dst.files.is_none() {
							let mut str = String::new();
							entry.read_to_string(&mut str)?;
							dst.files = Some(str);
						}
					}
					_ => {}
				}
			}
			Ok(())
		}).for_each(|state| {
			if let Err(err) = state {
				eprintln!("Error: {err}");
			}
		});

		Repo {
			created: Instant::now(),
			packages: packages.into_inner().unwrap(),
		}
	}
}

