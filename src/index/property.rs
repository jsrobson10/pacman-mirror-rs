use base64::{prelude::BASE64_STANDARD, Engine};
use rouille::Response;

use crate::database::repo::holder::RepoHolder;


pub enum PropertyType {
	PgpSig,
	Sha256,
}

pub fn get_property(repo_holder: &'static RepoHolder, file: &str, ty: PropertyType) -> anyhow::Result<Response> {
	let repo = repo_holder.get_or_refresh();
	let Some(package) = repo.packages.get(file) else {
		return Ok(Response::empty_404());
	};
	Ok(match ty {
		PropertyType::PgpSig => {
			let data = BASE64_STANDARD.decode(package.desc.pgpsig.as_ref())?;
			Response::from_data("application/pgp-signature", data)
		}
		PropertyType::Sha256 => {
			Response::text(format!("{}  {}", hex::encode(&package.desc.sha256sum), package.desc.filename.as_ref()))
		}
	})
}

