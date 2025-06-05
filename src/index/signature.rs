use std::path::PathBuf;

use anyhow::{anyhow, bail};
use rand::seq::{IndexedRandom, SliceRandom};
use rouille::Response;

use crate::{config::CONFIG, database::repo::holder::RepoHolder};


pub fn get_signature(repo_holder: &'static RepoHolder, file: &str) -> anyhow::Result<Response> {
	let Some(file) = file.strip_suffix(&*CONFIG.arch).map(|v| v.strip_suffix("-")).flatten() else {
		return Ok(Response::empty_404());
	};
	let repo = repo_holder.get_or_refresh();
	let Some(package) = repo.packages.get(file) else {
		return Ok(Response::empty_404());
	};

	let mut mirrors = package.mirrors.clone();
	mirrors.shuffle(&mut rand::rng());

	for mirror in mirrors {
		let url = format!("{}-{}.pkg.tar.zst.sig", PathBuf::from(mirror.get(&repo_holder.name)).join(file).to_string_lossy(), CONFIG.arch);
		let Ok(res) = minreq::get(&url).send() else {
			eprintln!("Error: {url} failed");
			continue;
		};
		if res.status_code != 200 {
			eprintln!("Error: {url} got {} {}", res.status_code, res.reason_phrase);
			continue;
		}
		return Ok(Response::from_data("application/pgp-signature", res.into_bytes()));
	}

	bail!("Out of mirrors");
}

