use std::{io::Read, sync::Arc};

use flate2::read::GzDecoder;
use itertools::Itertools;
use replay_buffer::{ReplayBuffer, ReplayBufferReader};

use crate::database::{desc::Desc, mirror_data::MirrorData};


impl MirrorData {

    pub fn update(&self) -> anyhow::Result<ReplayBufferReader<Arc<Desc>>> {

        let repo_url = self.repo_url.as_ref();
        let res = minreq::get(repo_url).send_lazy()?;
        
        if res.status_code != 200 {
            anyhow::bail!("Request {repo_url} failed with code {}: {}", res.status_code, res.reason_phrase);
        }

        let packages = ReplayBuffer::empty();
        let packages_writer = packages.write();
        
        *self.state.write().unwrap() = super::State {
            packages: packages.clone(),
        };

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
            packages_writer.push(desc);
        }
        Ok(packages.read())
    }
}

