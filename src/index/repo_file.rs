use itertools::Itertools;
use rouille::{Request, Response};

use super::get_database;


pub fn get_repo_file(req: &Request, repo: String, file: String) -> anyhow::Result<Response> {
	if let Some([_, end]) = file.split('.').collect_array().filter(|v| v[0] == repo) {
		match end {
			"db" => {
				return get_database(req, repo, false);
			}
			"files" => {
				return get_database(req, repo, true);
			}
			_ => {}
		}
	}
	todo!()
}
