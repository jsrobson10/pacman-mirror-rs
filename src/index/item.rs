use itertools::Itertools;
use rouille::Response;

use crate::database::DB;

use super::{get_database, get_package, get_property, property::PropertyType};


pub fn get_item(repo_name: String, file: String) -> anyhow::Result<Response> {
    let Some(repo) = DB.repos.get(repo_name.as_str()) else {
        return Ok(Response::empty_404());
    };
    if let Some([_, end]) = file.split('.').collect_array().filter(|v| v[0] == repo_name) {
        match end {
            "db" => {
                return get_database(repo, false);
            }
            "files" => {
                return get_database(repo, true);
            }
            _ => {
                return Ok(Response::empty_404());
            }
        }
    }
    if let Some(file) = file.strip_suffix(".sig") {
        return get_property(repo, file, PropertyType::PgpSig);
    }
    if let Some(file) = file.strip_suffix(".sha256") {
        return get_property(repo, file, PropertyType::Sha256);
    }
    get_package(repo, &file)
}

