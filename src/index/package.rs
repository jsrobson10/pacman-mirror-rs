use std::{io::{Cursor, Read, Write}, path::PathBuf, sync::{Arc, RwLockReadGuard}};
use anyhow::anyhow;
use rand::seq::SliceRandom;
use rouille::{Response, ResponseBody};

use crate::{cache::{Cache, CacheReader}, config::CONFIG, database::{mirror::Mirror, package::{DataSource, Package}, repo::holder::RepoHolder, DB}};


pub fn download_package(repo_holder: &'static RepoHolder, file: String, mut src: impl Read, mut dst: Cache<u8>) -> anyhow::Result<()> {
	let repo = repo_holder.get_without_refresh();
	let package = repo.packages.get(file.as_str()).ok_or_else(|| anyhow!("Package is None"))?;
	package.lock_and_set_source(DataSource::Partial(dst.reader()));

	let mut do_transfer = || -> std::io::Result<()> {
		let mut buf = [0u8; 1024];
		loop {
			let len = src.read(&mut buf)?;
			if len == 0 {
				break;
			}
			dst.write(&buf[..len])?;
		}
		Ok(())
	};
	if let Err(err) = do_transfer() {
		package.lock_and_set_source(DataSource::None);
		Err(err.into())
	}
	else {
		package.lock_and_set_source(DataSource::Memory(dst.into()));
		Ok(())
	}
}

pub fn get_package(repo_holder: &'static RepoHolder, file: &str) -> anyhow::Result<Response> {
	let Some(file) = file.strip_suffix(&*CONFIG.arch).map(|v| v.strip_suffix("-")).flatten() else {
		return Ok(Response::empty_404());
	};
	let repo = repo_holder.get_or_refresh();
	let Some(package) = repo.packages.get(file) else {
		return Ok(Response::empty_404());
	};

	let data_src = package.source.read().unwrap();
	let response_body: ResponseBody;

	match &*data_src {
		DataSource::None => {
			
			let cache = Cache::<u8>::new();
			let reader = cache.reader();
			let mut mirrors = package.mirrors.clone();
			mirrors.shuffle(&mut rand::rng());

			for mirror in mirrors {
				let url = format!("{}-{}.pkg.tar.zst", PathBuf::from(mirror.get(&repo_holder.name)).join(file).to_string_lossy(), CONFIG.arch);
				let Ok(res) = minreq::get(&url).send_lazy() else {
					eprintln!("Error: {url} failed");
					continue;
				};
				if res.status_code != 200 {
					eprintln!("Error: {url} got {} {}", res.status_code, res.reason_phrase);
					continue;
				}
				let file = file.to_owned();
				rayon::spawn(move || {
					if let Err(err) = download_package(repo_holder, file, res, cache) {
						eprintln!("Error: {err}");
					}
				});
				break;
			}
			response_body = ResponseBody::from_reader(reader);
		}
		DataSource::Partial(reader) => {
			let reader = reader.reader();
			response_body = match reader.get_size_hint() {
				Some(len) => ResponseBody::from_reader_and_size(reader, len),
				None => ResponseBody::from_reader(reader),
			};
		}
		DataSource::Memory(buff) => {
			response_body = ResponseBody::from_reader_and_size(Cursor::new(buff.clone()), buff.len());
		}
	}

	Ok(Response {
		status_code: 200,
		headers: vec![
			("Content-Type".into(), "application/x-tar".into()),
		],
		data: response_body,
		upgrade: None,
	})
}

