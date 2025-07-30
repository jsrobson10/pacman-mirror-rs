use maud::html;
use rouille::{Request, Response};

use crate::Index;

use super::template;


impl Index {
    pub fn get_repo_list(&self, req: &Request) -> Response {
        Response::html(template(req.raw_url(), html! {
            ul {
                @for repo in self.config.repos.iter() {
                    li { a href=(repo) { (repo) } }
                }
            }
        }))
    }
}
