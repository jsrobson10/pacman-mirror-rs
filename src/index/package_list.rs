use std::borrow::Cow;

use itertools::Itertools;
use maud::html;
use rouille::{Request, Response};

use crate::{cache::DataSource, config::CONFIG, database::DB};

use super::template;


pub fn get_package_list(req: &Request, repo: String) -> anyhow::Result<Response> {
	let Some(repo_holder) = DB.repos.get(repo.as_str()) else {
		return Ok(Response::empty_404());
	};
	let repo = repo_holder.get_or_refresh();
	let mut pkgs = repo.packages.values().map(|v| {
		(v.desc.name.as_ref(), v.desc.filename.as_ref(), v.desc.version.as_ref(), v.mirrors.len(), match v.cache.get() {
			DataSource::Empty => {
				Cow::Borrowed("-")
			}
			DataSource::Partial(v) => {
				if let Some(size_hint) = v.get_size_hint() {
					let progress = (v.get_buffer_size() as f64) / (size_hint as f64) * 100.0;
					Cow::Owned(format!("Partial ({progress:.2})"))
				}
				else {
					Cow::Borrowed("Partial")
				}
			}
			DataSource::Memory(_) => {
				Cow::Borrowed("Full")
			}
		})
	}).collect_vec();
	pkgs.sort_by(|a,b| a.0.cmp(b.0));

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
					td { a href=(filename) { (name) } " (" a href={ (filename) ".sig" } { "sig" } ")" }
					td { (version) }
					td { (mirror_count) " / " (CONFIG.mirrors.len()) }
					td { (cache_state) }
				}
			}
		}
	})))
}

