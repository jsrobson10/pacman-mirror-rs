use itertools::Itertools;
use rouille::Response;

use crate::database::DB;

use super::{get_database, get_package, get_signature};


pub fn get_item(repo_name: String, file: String) -> anyhow::Result<Response> {
	let Some(repo) = DB.repos.get(repo_name.as_str()) else {
		return Ok(Response::empty_404());
	};
	if let Some([_, end]) = file.split('.').collect_array().filter(|v| v[0] == repo_name) {
		match end {
			"db" => {
				return get_database(repo, false);
			}
			"files" => {
				return get_database(repo, true);
			}
			_ => {
				return Ok(Response::empty_404());
			}
		}
	}
	if let Some(file) = file.strip_suffix(".pkg.tar.zst") {
		return get_package(repo, file);
	}
	if let Some(file) = file.strip_suffix(".pkg.tar.zst.sig") {
		return get_signature(repo, file);
	}
	Ok(Response::empty_404())
}

