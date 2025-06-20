use std::{collections::HashMap, io::Write, sync::Arc, time::{Duration, SystemTime}};

use owning_ref::ArcRef;
use parser::ParseError;


pub mod parser;
pub mod writer;
mod cursor;

pub struct Desc {
    data: HashMap<ArcRef<str>, ArcRef<str>>,
    pub name: ArcRef<str>,
    pub filename: ArcRef<str>,
    pub version: ArcRef<str>,
    pub pgpsig: ArcRef<str>,
    pub builddate: SystemTime,
    pub sha256sum: [u8; 32],
    pub csize: usize,
}

impl Desc {
    pub fn parse(src: Arc<str>) -> Result<Desc, ParseError> {
        let mut data = HashMap::new();
        parser::parse(ArcRef::new(src), |k, v| {
            data.insert(k, v);
        })?;
        let (
            Some(name),
            Some(filename),
            Some(version),
            Some(pgpsig),
            Some(p_sha256sum),
            Some(p_builddate),
            Some(p_csize),
        ) = (
            data.get("NAME").cloned(),
            data.get("FILENAME").cloned(),
            data.get("VERSION").cloned(),
            data.get("PGPSIG").cloned(),
            data.get("SHA256SUM").cloned(),
            data.get("BUILDDATE"),
            data.get("CSIZE"),
        ) else {
            return Err(ParseError::MissingFields);
        };
        let builddate = match p_builddate.parse() {
            Ok(secs) => SystemTime::UNIX_EPOCH + Duration::from_secs(secs),
            Err(err) => return Err(ParseError::ParseInt { field: "BUILDDATE", err }),
        };
        let csize: usize = match p_csize.parse() {
            Ok(len) => len,
            Err(err) => return Err(ParseError::ParseInt { field: "CSIZE", err }),
        };
        let mut sha256sum = [0u8; 32];
        if let Err(err) = hex::decode_to_slice(p_sha256sum.as_ref(), &mut sha256sum) {
            return Err(ParseError::Decode { field: "SHA256SUM", err });
        }
        Ok(Self { data, name, filename, version, pgpsig, builddate, sha256sum, csize })
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

