use std::{collections::HashMap, ffi::OsStr, io::{BufRead, BufReader, Read, Write}, path::PathBuf, sync::Mutex, time::SystemTime};

use flate2::read::{DeflateDecoder, GzDecoder};
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{config::CONFIG, database::desc::{self, DescParser}};

use super::package::Package;

#[derive(Debug)]
pub struct Repo {
	pub name: String,
	pub created: SystemTime,
	pub packages: HashMap<String, Package>,
}

impl Repo {
	pub fn load_all(name: String) -> anyhow::Result<Repo> {

		let packages: Mutex<HashMap<String, Package>> = Mutex::new(HashMap::new());
		
		CONFIG.mirrors.par_iter().try_for_each(|mirror| -> anyhow::Result<()> {
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
				
				let mut packages = packages.lock().unwrap();
				let dst = packages.entry(pkg_name.into()).or_insert_with(|| Package::new(pkg_name.into()));
				dst.mirrors.insert(mirror.clone());

				match ty {
					"desc" => {
						let desc_parser = DescParser::new(BufReader::new(entry));

						if dst.desc.len() == 0 {
							for res in desc_parser {
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
				
//				std::io::stdout().lock().write_fmt(format_args!("====== LOCK ======>\nFound: {} => {}\n<==== UNLOCK =====\n", path, String::from_utf8(bytes?)?))?;
			}
			Ok(())
		})?;

		Ok(Repo {
			name,
			created: SystemTime::now(),
			packages: packages.into_inner().unwrap(),
		})
	}
}

