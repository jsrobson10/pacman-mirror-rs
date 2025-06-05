use std::{collections::HashMap, io::{BufReader, Read}, path::PathBuf, sync::{Arc, Mutex}, time::Instant};
use flate2::read::GzDecoder;
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use crate::{config::{self, CONFIG}, database::desc::DescParser};

use super::package::Package;

#[derive(Debug)]
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
			let url = [mirror.get(&name), format!("{name}.files")].into_iter().collect::<PathBuf>().to_string_lossy().into_owned();
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
						if dst.desc.len() == 0 {
							for res in DescParser::new(BufReader::new(entry)) {
								let (field, body) = res?;
								dst.desc.insert(field, body);
							}
						}
					}
					"files" => {
						if dst.files.len() > 0 {
							entry.read_to_string(&mut dst.files)?;
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

