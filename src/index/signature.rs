use rouille::Response;

use crate::database::repo::holder::RepoHolder;


pub fn get_signature(repo: &'static RepoHolder, file: &str) -> anyhow::Result<Response> {
	let repo = repo.get_or_refresh();
	let Some(package) = repo.packages.get(file) else {
		return Ok(Response::empty_404());
	};

	todo!()
}

