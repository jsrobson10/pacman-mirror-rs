use std::{io::{Cursor, Read, Write}, path::PathBuf, sync::Arc};
use anyhow::{anyhow, bail};
use log::{error, info, warn};
use rand::seq::SliceRandom;
use rouille::{Response, ResponseBody};
use sha2::{Digest, Sha256};

use crate::{cache::{self, DataSource}, database::repo::holder::RepoHolder, Index};


pub fn download_package(repo_holder: Arc<RepoHolder>, file: String, mut src: impl Read, mut dst: cache::partial::Writer) -> anyhow::Result<()> {
    let repo = repo_holder.get_without_refresh();
    let package = repo.packages.get(file.as_str()).ok_or_else(|| anyhow!("Package is None"))?;
    let dst_source = dst.source().clone();

    package.cache.set(DataSource::Partial(dst_source.clone()));

    let mut do_transfer = || -> std::io::Result<()> {
        let mut buffer = [0u8; 16384];
        while let len@1.. = src.read(&mut buffer)? {
            dst.write_all(&buffer[..len])?;
            dst.flush()?;
        }
        Ok(())
    };
    if let Err(err) = do_transfer() {
        package.cache.set(DataSource::Empty);
        return Err(err.into());
    }
    if Sha256::digest(&*dst_source.data.read().unwrap()).as_slice() != package.desc.sha256sum {
        package.cache.set(DataSource::Empty);
        bail!("Checksums do not match");
    }
    Ok(())
}

impl Index {
    pub fn get_package(&self, repo_holder: Arc<RepoHolder>, file: &str) -> anyhow::Result<Response> {
        let repo = repo_holder.get_without_refresh();
        let Some(package) = repo.packages.get(file) else {
            return Ok(Response::empty_404());
        };
        let response_body = match package.cache.get() {
            DataSource::Empty => {
                let mut mirrors = package.mirrors.clone();
                let writer = cache::partial::Writer::new();
                let reader = writer.source().reader();
                mirrors.shuffle(&mut rand::rng());
    
                for mirror in mirrors {
                    let url = PathBuf::from(mirror.get(&self.config, &repo_holder.name)).join(file);
                    let url_str = url.to_string_lossy();
                    let Ok(res) = minreq::get(&*url_str).send_lazy() else {
                        warn!("{url_str} failed");
                        continue;
                    };
                    if res.status_code != 200 {
                        warn!("{url_str} got {} {}", res.status_code, res.reason_phrase);
                        continue;
                    }
                    let file = file.to_owned();
                    let repo_holder = repo_holder.clone();
                    std::thread::spawn(move || {
                        info!("Started download: {}", url.to_string_lossy());
                        if let Err(err) = download_package(repo_holder, file, res, writer) {
                            error!("{err}");
                            return;
                        }
                        info!("Download complete: {}", url.to_string_lossy());
                    });
                    break;
                }
                ResponseBody::from_reader_and_size(reader, package.desc.csize)
            }
            DataSource::Partial(source) => {
                let reader = source.reader();
                ResponseBody::from_reader_and_size(reader, package.desc.csize)
            }
            DataSource::Memory(buff) => {
                ResponseBody::from_reader_and_size(Cursor::new(buff.clone()), buff.len())
            }
        };
        Ok(Response {
            status_code: 200,
            headers: vec![
                ("Content-Type".into(), "application/x-tar".into()),
            ],
            data: response_body.with_chunked_threshold(usize::max_value()),
            upgrade: None,
        })
    }
}

