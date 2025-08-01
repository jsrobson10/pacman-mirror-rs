use std::{ffi::OsString, io::Read, path::Path, sync::Arc};

use flate2::read::GzDecoder;
use log::{debug, trace};
use replay_buffer::ReplayBufferWriter;

use crate::database::{desc::Desc, mirror_data::MirrorData};


struct PartialPackage {
    name: OsString,
    desc: Option<Desc>,
    files: Option<Arc<str>>,
}

impl PartialPackage {
    pub fn new(name: OsString) -> Self {
        Self { name, desc: None, files: None }
    }
    pub fn into_desc(self, files: bool) -> Option<Arc<Desc>> {
        let mut desc = self.desc?;
        if files {
            desc.files = Some(self.files?);
        }
        Some(desc.into())
    }
}

impl MirrorData {

    pub fn prepare_for_update(&self) -> ReplayBufferWriter<Arc<Desc>> {
        let repo_url = &self.repo_url;
        trace!("Prepare for connection: {repo_url}");
        let packages_writer = ReplayBufferWriter::new();
        *self.state.write().unwrap() = super::State {
            packages: packages_writer.source().clone(),
        };
        packages_writer
    }

    pub fn update(&self, dst: &mut ReplayBufferWriter<Arc<Desc>>, files: bool) -> anyhow::Result<()> {

        let repo_url = self.repo_url.as_ref();
        let db_url_path = Path::new(repo_url)
            .join(format!("{}.{}", self.repo_name, match files {
                true => "files",
                false => "db",
            }));
        let db_url = db_url_path.to_string_lossy();
        let res = minreq::get(db_url.as_ref()).send_lazy()?;

        if res.status_code != 200 {
            anyhow::bail!("Request {repo_url} failed with code {}: {}", res.status_code, res.reason_phrase);
        }
        
        debug!("Started connection: {repo_url}");

        let mut partial_pkg = Option::<PartialPackage>::None;
        let mut archive = tar::Archive::new(GzDecoder::new(res));

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.into_owned();
            let mut path_iter = path.iter();

            let name = match path_iter.next() {
                None => continue,
                Some(v) => v,
            };
            let ty = match path_iter.next() {
                None => continue,
                Some(v) => v,
            };
            let partial_pkg = partial_pkg.get_or_insert_with(|| PartialPackage::new(name.into()));
            if partial_pkg.name != name {
                let pkg = std::mem::replace(partial_pkg, PartialPackage::new(name.into()));
                if let Some(desc) = pkg.into_desc(files) {
                    dst.push(desc);
                }
            }
            if path_iter.next().is_some() {
                continue;
            }
            let mut str = String::new();
            if ty == "desc" {
                entry.read_to_string(&mut str)?;
                partial_pkg.desc = Some(Desc::parse(str.into())?);
            }
            else if ty == "files" {
                entry.read_to_string(&mut str)?;
                partial_pkg.files = Some(str.into());
            }
        }
        if let Some(desc) = partial_pkg.take().and_then(|v| v.into_desc(files)) {
            dst.push(desc);
        }
        debug!("Wrapping up: {repo_url}");
        Ok(())
    }
}

