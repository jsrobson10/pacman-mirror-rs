use std::sync::Arc;

use itertools::Itertools;
use rouille::Response;

use crate::Index;

use super::{get_property, property::PropertyType};


impl Index {
    pub fn get_item(self: &Arc<Self>, repo_name: Arc<str>, file: Arc<str>) -> anyhow::Result<Response> {
        let Some(repo) = self.db.repos.get(repo_name.as_ref()).cloned() else {
            return Ok(Response::empty_404());
        };
        if let Some([_, end]) = file.split('.').collect_array().filter(|v| v[0] == repo_name.as_ref()) {
            return match end {
                "db" => self.get_database(repo, true),
                "files" => self.get_database(repo, false),
                _ => Ok(Response::empty_404()),
            }
        }
        if let Some(file) = file.strip_suffix(".sig") {
            return get_property(repo, file, PropertyType::PgpSig);
        }
        if let Some(file) = file.strip_suffix(".sha256") {
            return get_property(repo, file, PropertyType::Sha256);
        }
        self.get_package(repo, file)
    }
}

