use config::CONFIG;
use error::check;
use rouille::router;

mod index;
pub mod config;
pub mod error;
pub mod database;

fn main() -> anyhow::Result<()> {

	let config = &*CONFIG;
	println!("Loaded config: {config:#?}");

	rouille::start_server("localhost:8080", |req| {
		router!(req,
			(GET) (/) => { index::get_root(req) },
			(GET) (/{repo: String}) => { index::get_repo_root(req, repo).unwrap() },
			_ => rouille::Response::empty_404()
		)
	});
}

