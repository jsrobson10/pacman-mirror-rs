use std::sync::Arc;

use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use crate::database::mirror::Mirror;


#[derive(Debug,Serialize,Deserialize)]
pub struct Config {
	pub name: String,
	pub arch: String,
	pub repos: Vec<String>,
	pub mirrors: Vec<Arc<Mirror>>,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			name: "archlinux".into(),
			arch: "x86_64".into(),
			repos: vec!["core".into(), "multilib".into(), "extra".into()],
			mirrors: vec![Arc::new(Mirror::new("https://geo.mirror.pkgbuild.com/$repo/os/$arch/".into()))],
		}
	}
}

impl Config {
	fn try_load(path: &str) -> anyhow::Result<Self> {
		let data = std::fs::read_to_string(path)?;
		Ok(serde_yml::from_str(&data)?)
	}
	pub fn load(path: &str) -> anyhow::Result<Self> {
		if let Ok(cfg) = Self::try_load(path) {
			return Ok(cfg);
		}
		let cfg = Self::default();
		std::fs::write(path, serde_yml::to_string(&cfg)?)?;
		Ok(cfg)
	}
}

lazy_static! {
	pub static ref CONFIG: Config = Config::load("config.yml").unwrap();
}

