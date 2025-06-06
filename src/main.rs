use config::CONFIG;
use env_logger::Env;
use log::{debug, info};
use rouille::router;

mod index;
pub mod vercmp;
pub mod cache;
pub mod config;
pub mod error;
pub mod database;

fn main() -> anyhow::Result<()> {
	env_logger::builder()
		.parse_env(Env::new().filter_or("RUST_LOG", "info"))
		.try_init()?;
	
	let config = &*CONFIG;
	debug!("Loaded config: {config:#?}");
	info!("Listening on {}", CONFIG.listen.as_ref());

	rouille::start_server(CONFIG.listen.as_ref(), |req| {
		debug!("{}: {}", req.method(), req.raw_url());
		router!(req,
			(GET) (/) => { index::get_repo_list(req) },
			(GET) (/{repo: String}) => { rouille::Response::redirect_301(format!("/{repo}/")) },
			(GET) (/{repo: String}/) => { index::get_package_list(req, repo).unwrap() },
			(GET) (/{repo: String}/{file: String}) => { index::get_item(repo, file).unwrap() },
			_ => rouille::Response::empty_404()
		)
	});
}

