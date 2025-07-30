use std::{sync::Arc, time::Duration};

use serde::{Serialize, Deserialize};
use crate::database::mirror::Mirror;


#[derive(Debug,Serialize,Deserialize)]
pub struct Config {
    pub name: Arc<str>,
    pub listen: Arc<str>,
    pub arch: Arc<str>,
    pub timeout: Duration,
    pub repos: Vec<Arc<str>>,
    pub mirrors: Vec<Mirror>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "archlinux".into(),
            listen: "localhost:8080".into(),
            arch: "x86_64".into(),
            timeout: Duration::from_secs(3600),
            repos: vec!["core".into(), "multilib".into(), "extra".into()],
            mirrors: vec![Mirror::new("https://geo.mirror.pkgbuild.com/$repo/os/$arch/".into())],
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

