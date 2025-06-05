pub mod database;
pub mod package;
pub mod signature;
pub mod repo_list;
pub mod package_list;
pub mod item;

use maud::html;

pub use {
	package::get_package,
	signature::get_signature,
	database::get_database,
	repo_list::get_repo_list,
	package_list::get_package_list,
	item::get_item,
};

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

