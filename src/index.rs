pub mod database;
pub mod package;
pub mod property;
pub mod repo_list;
pub mod package_list;
pub mod item;

use std::sync::Arc;

use maud::html;

use crate::{Config, Database};

pub use property::get_property;

pub struct Index {
    db: Arc<Database>,
    config: Arc<Config>,
}

impl Index {
    pub fn new(db: Arc<Database>) -> Self {
        let config = db.config.clone();
        Self { db, config }
    }
}

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

