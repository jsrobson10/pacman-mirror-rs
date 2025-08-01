use std::{io::{Read, Write}, path::Path, sync::Arc};
use anyhow::{anyhow, bail};
use log::{debug, error, info, warn};
use rand::seq::SliceRandom;
use replay_buffer::ReplayBufferWriter;
use rouille::{Response, ResponseBody};
use sha2::Digest;

use crate::{cache::DataSource, database::Repo, Index};


pub fn download_package(repo: Arc<Repo>, name: Arc<str>, mut src: impl Read, mut dst: ReplayBufferWriter<u8>) -> anyhow::Result<()> {
    let repo_state = repo.state.read().unwrap();
    let package = repo_state.packages.get(name.as_ref()).ok_or_else(|| anyhow!("Package {name} is None"))?;
    let mut hasher = sha2::Sha256::new();

    package.cache.set(DataSource::Memory(dst.source().clone()));

    let mut do_transfer = || -> std::io::Result<()> {
        let mut buffer = [0u8; 16384];
        while let len@1.. = src.read(&mut buffer)? {
            let buf = &buffer[..len];
            hasher.write_all(buf)?;
            hasher.flush()?;
            dst.write_all(buf)?;
            dst.flush()?;
        }
        Ok(())
    };
    if let Err(err) = do_transfer() {
        package.cache.set(DataSource::Empty);
        return Err(err.into());
    }
    let digest = hasher.finalize();
    if digest.as_slice() != package.desc.sha256sum {
        package.cache.set(DataSource::Empty);
        bail!("Checksums do not match");
    }
    debug!("done transferring file: {} ({})", name, hex::encode(digest.as_slice()));

    Ok(())
}

impl Index {
    pub fn get_package(&self, repo: Arc<Repo>, file: Arc<str>) -> anyhow::Result<Response> {
        let repo_state = repo.state.read().unwrap();
        let package_name = repo_state.packages_by_filename.get(file.as_ref());
        let Some(package) = package_name.and_then(|v| repo_state.packages.get(v.as_ref())) else {
            return Ok(Response::empty_404());
        };
        let response_body = match package.cache.get() {
            DataSource::Empty => {
                let mut mirrors = package.mirrors.clone();
                let cache = ReplayBufferWriter::new();
                let reader = cache.source().read();
                mirrors.shuffle(&mut rand::rng());
    
                for mirror in mirrors {
                    let url = Path::new(mirror.as_ref()).join(file.as_ref());
                    let url_str = url.to_string_lossy();
                    let Ok(res) = minreq::get(&*url_str).send_lazy() else {
                        warn!("{url_str} failed");
                        continue;
                    };
                    if res.status_code != 200 {
                        warn!("{url_str} got {} {}", res.status_code, res.reason_phrase);
                        continue;
                    }
                    let (name, repo, cache) = (package.desc.name.clone(), repo.clone(), cache);
                    std::thread::spawn(move || {
                        info!("Started download: {}", url.to_string_lossy());
                        if let Err(err) = download_package(repo, name, res, cache) {
                            error!("{err}");
                            return;
                        }
                        info!("Download complete: {}", url.to_string_lossy());
                    });
                    break;
                }
                ResponseBody::from_reader_and_size(reader, package.desc.csize)
            }
            DataSource::Memory(source) => {
                let reader = source.read();
                ResponseBody::from_reader_and_size(reader, package.desc.csize)
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

