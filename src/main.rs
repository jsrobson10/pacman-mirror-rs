use config::CONFIG;
use rouille::router;

mod index;
pub mod cache;
pub mod config;
pub mod error;
pub mod database;

fn main() -> anyhow::Result<()> {
	let config = &*CONFIG;
	println!("Loaded config: {config:#?}");

	rouille::start_server("localhost:8080", |req| {
		router!(req,
			(GET) (/) => { index::get_repo_list(req) },
			(GET) (/{repo: String}) => { index::get_package_list(req, repo).unwrap() },
			(GET) (/{repo: String}/{file: String}) => { index::get_item(repo, file).unwrap() },
			_ => rouille::Response::empty_404()
		)
	});
}

