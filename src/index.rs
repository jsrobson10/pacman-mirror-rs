use itertools::Itertools;
use maud::html;
use rouille::{Request, Response};

use crate::database::{repo::Repo, DB};

fn template(path: &str, body: maud::Markup) -> maud::Markup {
	html! {
		(maud::DOCTYPE)
		head {
			title { "Index of " (path) }
			style { r#"
			th, td {
				padding: 0 1.5em;
			}
			table {
				border: 1px black solid;
			}
			"# }
		}
		body {
			h1 { "Index of " (path) }
			(body)
		}
	}
}


pub fn get_root(req: &Request) -> Response {
	Response::html(template(req.raw_url(), html! {
		ul {
			@for repo in crate::config::CONFIG.repos.iter() {
				li { a href=(repo) { (repo) } }
			}
		}
	}))
}

pub fn get_repo_root(req: &Request, repo: String) -> anyhow::Result<Response> {
	let Some(repo) = DB.get_or_refresh(&repo) else {
		return Ok(rouille::Response::empty_404());
	};
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

