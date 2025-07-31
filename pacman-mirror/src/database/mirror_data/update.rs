use std::{io::Read, sync::Arc};

use flate2::read::GzDecoder;
use itertools::Itertools;
use log::trace;
use replay_buffer::ReplayBufferWriter;

use crate::database::{desc::Desc, mirror_data::MirrorData};


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

    pub fn update(&self, dst: &mut ReplayBufferWriter<Arc<Desc>>) -> anyhow::Result<()> {

        let repo_url = self.repo_url.as_ref();
        let res = minreq::get(self.db_url.as_ref()).send_lazy()?;

        if res.status_code != 200 {
            anyhow::bail!("Request {repo_url} failed with code {}: {}", res.status_code, res.reason_phrase);
        }
        
        trace!("Started connection: {repo_url}");

        let mut archive = tar::Archive::new(GzDecoder::new(res));
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.into_owned();
            let [_, ty] = match path.iter().flat_map(|v| v.to_str()).collect_array() {
                None => continue,
                Some(v) => v,
            };
            if ty != "desc" {
                continue;
            }
            let desc = {
                let mut dst = String::new();
                entry.read_to_string(&mut dst)?;
                Desc::parse(dst.into())?
            };
            dst.push(desc);
        }
        Ok(())
    }
}

