use std::{io::Write, sync::Arc, time::{Duration, SystemTime}};

use itertools::Itertools;
use parser::ParseError;


pub mod parser;
pub mod writer;
mod cursor;

#[derive(Clone)]
pub struct Desc {
    fields: Vec<(Arc<str>, Arc<str>)>,
    pub files: Option<Arc<str>>,
    pub name: Arc<str>,
    pub filename: Arc<str>,
    pub version: Arc<str>,
    pub pgpsig: Arc<str>,
    pub builddate: SystemTime,
    pub sha256sum: [u8; 32],
    pub csize: usize,
}

enum Known {
    Name, Filename, Version, PgpSig, Sha256Sum, BuildDate, Csize,
}

impl Desc {
    pub fn parse(src: Box<str>) -> Result<Desc, ParseError> {
        let mut data = Vec::<(Arc<str>, Arc<str>)>::new();
        parser::parse(src, |k, v| {
            data.push((k.to_uppercase().into(), v.into()));
        })?;
        let mut known: [Option<Arc<str>>; 7] = std::array::from_fn(|_| None);
        for (k, v) in data.iter() {
            known[match k.as_ref() {
                "NAME" => Known::Name,
                "FILENAME" => Known::Filename,
                "VERSION" => Known::Version,
                "PGPSIG" => Known::PgpSig,
                "SHA256SUM" => Known::Sha256Sum,
                "BUILDDATE" => Known::BuildDate,
                "CSIZE" => Known::Csize,
                _ => continue,
            } as usize] = Some(v.clone());
        }
        let known: Vec<Arc<str>> = known.into_iter().map(|v| v.ok_or(ParseError::MissingFields)).try_collect()?;
        let mut sha256sum = [0u8; 32];
        if let Err(err) = hex::decode_to_slice(known[Known::Sha256Sum as usize].as_ref(), &mut sha256sum) {
            return Err(ParseError::Decode { field: "SHA256SUM", err });
        }
        Ok(Self {
            files: None,
            fields: data,
            name: known[Known::Name as usize].clone(),
            filename: known[Known::Filename as usize].clone(),
            version: known[Known::Version as usize].clone(),
            pgpsig: known[Known::PgpSig as usize].clone(),
            builddate: match known[Known::BuildDate as usize].parse() {
                Ok(secs) => SystemTime::UNIX_EPOCH + Duration::from_secs(secs),
                Err(err) => return Err(ParseError::ParseInt { field: "BUILDDATE", err }),
            },
            sha256sum,
            csize: match known[Known::Csize as usize].parse() {
                Ok(len) => len,
                Err(err) => return Err(ParseError::ParseInt { field: "CSIZE", err }),
            },
        })
    }
    pub fn write_to(&self, dst: impl Write) -> std::io::Result<()> {
        writer::write(dst, self.fields.iter().map(|(k, v)| (&**k, &**v)))
    }
    pub fn write_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut dst = Vec::new();
        self.write_to(&mut dst)?;
        Ok(dst)
    }
}

