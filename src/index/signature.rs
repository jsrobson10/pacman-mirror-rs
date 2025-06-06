use base64::{prelude::BASE64_STANDARD, Engine};
use rouille::Response;

use crate::database::repo::holder::RepoHolder;


pub fn get_signature(repo_holder: &'static RepoHolder, file: &str) -> anyhow::Result<Response> {
	let repo = repo_holder.get_or_refresh();
	let Some(package) = repo.packages.get(file) else {
		return Ok(Response::empty_404());
	};
	let pgpsig = BASE64_STANDARD.decode(package.desc.pgpsig.as_ref())?;
	Ok(Response::from_data("application/pgp-signature", pgpsig))
}

