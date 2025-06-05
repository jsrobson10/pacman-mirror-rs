use itertools::Itertools;
use maud::html;
use rouille::{Request, Response};

use crate::database::DB;

use super::template;


pub fn get_package_list(req: &Request, repo: String) -> anyhow::Result<Response> {
	let Some(repo_holder) = DB.repos.get(repo.as_str()) else {
		return Ok(Response::empty_404());
	};
	let repo = repo_holder.get_or_refresh();
	let mut pkgs = repo.packages.iter().map(|(_,v)| &v.name).collect_vec();
	pkgs.sort();

	Ok(Response::html(template(req.raw_url(), html! {
		ul {
			@for pkg in pkgs {
				li { (pkg) }
			}
		}
	})))
}
