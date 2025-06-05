pub mod root;
pub mod repo_root;
pub mod repo_file;
pub mod database;

use maud::html;

pub use repo_file::get_repo_file;
pub use repo_root::get_repo_root;
pub use database::get_database;
pub use root::get_root;

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

