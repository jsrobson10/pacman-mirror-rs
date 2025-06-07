use itertools::Itertools;
use maud::html;
use rouille::{Request, Response};

use crate::{cache::DataSource, config::CONFIG, database::DB, vercmp};

use super::template;


pub fn get_package_list(req: &Request, repo: String) -> anyhow::Result<Response> {
	let Some(repo_holder) = DB.repos.get(repo.as_str()) else {
		return Ok(Response::empty_404());
	};
	let repo = repo_holder.get_or_refresh();
	let mut pkgs = repo.packages.values().map(|v| {
		(v.desc.name.as_ref(), v.desc.filename.as_ref(), v.desc.version.as_ref(), v.mirrors.len(), match v.cache.get() {
			DataSource::Empty => {
				"-"
			}
			DataSource::Partial(_) => {
				"Partial"
			}
			DataSource::Memory(_) => {
				"Memory"
			}
		})
	}).collect_vec();
	pkgs.sort_by(|a,b| {
		a.0.cmp(b.0).then_with(|| {
			vercmp::alpm_pkg_ver_cmp(b.2, a.2)
		})
	});

	Ok(Response::html(template(req.raw_url(), html! {
		table {
			tr {
				th { "Name" }
				th { "Version" }
				th { "Mirrors" }
				th { "Cache State" }
			}
			@for (name, filename, version, mirror_count, cache_state) in pkgs {
				tr {
					td {
						a href=(filename) { (name) }
						" (" a href={ (filename) ".sha256" } { "hash" } ")"
						" (" a href={ (filename) ".sig" } { "sig" } ")"
					}
					td { (version) }
					td { (mirror_count) " / " (CONFIG.mirrors.len()) }
					td { (cache_state) }
				}
			}
		}
	})))
}

