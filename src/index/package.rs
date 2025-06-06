use std::{io::{Cursor, Read, Write}, path::PathBuf};
use anyhow::anyhow;
use log::{debug, error, info, warn};
use rand::seq::SliceRandom;
use rouille::{Response, ResponseBody};

use crate::{cache::{DataSource, PartialCache}, config::CONFIG, database::repo::holder::RepoHolder};


pub fn download_package(repo_holder: &'static RepoHolder, file: String, mut src: impl Read, mut dst: PartialCache<u8>) -> anyhow::Result<()> {
	let repo = repo_holder.get_without_refresh();
	let package = repo.packages.get(file.as_str()).ok_or_else(|| anyhow!("Package is None"))?;


	package.cache.set(DataSource::Partial(dst.reader()));

	let mut do_transfer = || -> std::io::Result<()> {
		let mut buf = [0u8; 65536];
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
		package.cache.set(DataSource::Empty);
		Err(err.into())
	}
	else {
		package.cache.set(DataSource::Memory(dst.into()));
		Ok(())
	}
}

pub fn get_package(repo_holder: &'static RepoHolder, file: &str) -> anyhow::Result<Response> {
	let repo = repo_holder.get_or_refresh();
	let Some(package) = repo.packages.get(file) else {
		return Ok(Response::empty_404());
	};
	let response_body: ResponseBody;

	match package.cache.get() {
		DataSource::Empty => {
			let cache = PartialCache::<u8>::with_capacity(package.desc.csize);
			let reader = cache.reader();
			let mut mirrors = package.mirrors.clone();
			mirrors.shuffle(&mut rand::rng());

			for mirror in mirrors {
				let url = PathBuf::from(mirror.get(&repo_holder.name)).join(file);
				let url_str = url.to_string_lossy();
				let Ok(res) = minreq::get(&*url_str).send_lazy() else {
					warn!("{url_str} failed");
					continue;
				};
				if res.status_code != 200 {
					warn!("{url_str} got {} {}", res.status_code, res.reason_phrase);
					continue;
				}
				let file = file.to_owned();
				std::thread::spawn(move || {
					debug!("Download started: {}", url.to_string_lossy());
					if let Err(err) = download_package(repo_holder, file, res, cache) {
						error!("{err}");
						return;
					}
					info!("Download: {}", url.to_string_lossy());
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

