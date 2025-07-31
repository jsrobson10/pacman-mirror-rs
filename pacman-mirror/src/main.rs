use std::sync::Arc;

use env_logger::Env;
use log::{debug, info};
use rouille::router;
pub use config::Config;
pub use database::Database;
pub use index::Index;

mod index;
mod cache;
mod config;
mod database;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .parse_env(Env::new().filter_or("RUST_LOG", "info"))
        .try_init()?;
    
    let config = Arc::new(Config::load("config.yaml")?);
    let database = Arc::new(Database::new(config.clone()));
    let index = Arc::new(Index::new(database.clone()));

    debug!("Loaded config: {config:#?}");
    info!("Listening on {}", config.listen.as_ref());

    rouille::start_server(config.listen.as_ref(), move |req| {
        debug!("{}: {}", req.method(), req.raw_url());
        router!(req,
            (GET) (/) => { index.get_repo_list(req) },
            (GET) (/{repo: String}) => { rouille::Response::redirect_301(format!("/{repo}/")) },
            (GET) (/{repo: String}/) => { index.get_package_list(req, repo).unwrap() },
            (GET) (/{repo: String}/{file: String}) => { index.get_item(repo.into(), file.into()).unwrap() },
            _ => rouille::Response::empty_404()
        )
    });
}

