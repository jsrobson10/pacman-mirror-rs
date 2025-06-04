use std::{collections::HashMap, sync::RwLock};
use lazy_static::lazy_static;
use repo::Repo;


pub mod desc;
pub mod mirror;
pub mod package;
pub mod repo;

#[derive(Default)]
pub struct Database {
	pub repos: HashMap<String, Repo>,
}

impl Database {
	
}

lazy_static! {
	pub static ref DB: RwLock<Database> = RwLock::new(Database::default());
}

