use maud::html;
use rouille::{Request, Response};

use crate::config::CONFIG;

use super::template;


pub fn get_root(req: &Request) -> Response {
	Response::html(template(req.raw_url(), html! {
		ul {
			@for repo in CONFIG.repos.iter() {
				li { a href=(repo) { (repo) } }
			}
		}
	}))
}
