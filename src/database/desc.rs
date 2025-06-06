use std::{collections::HashMap, io::Write, sync::Arc};

use owning_ref::ArcRef;
use parser::ParseError;


pub mod parser;
pub mod writer;
mod cursor;

pub struct Desc {
	data: HashMap<ArcRef<str>, ArcRef<str>>,
}

impl Desc {
	pub fn parse(src: impl Into<Arc<str>>) -> Result<Desc, ParseError> {
		let mut data = HashMap::new();
		parser::parse(ArcRef::new(src.into()), |k, v| {
			data.insert(k, v);
		})?;
		Ok(Self { data })
	}
	pub fn write_to(&self, dst: impl Write) -> std::io::Result<()> {
		writer::write(dst, self.data.iter().map(|(k, v)| (&**k, &**v)))
	}
	pub fn write_to_vec(&self) -> std::io::Result<Vec<u8>> {
		let mut dst = Vec::new();
		self.write_to(&mut dst)?;
		Ok(dst)
	}
	pub fn get(&self, name: &str) -> Option<&str> {
		self.data.get(name).map(|v| &**v)
	}
}

