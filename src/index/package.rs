use std::{io::{Cursor, Read, Write}, path::PathBuf, sync::{mpsc, Arc}};
use anyhow::{anyhow, bail};
use log::{error, info, warn};
use rand::seq::SliceRandom;
use rouille::{Response, ResponseBody};
use sha2::{Digest, Sha256};

use crate::{cache::{partial::PartialCacheWriter, DataSource}, database::repo::holder::RepoHolder};


pub fn download_package(repo_holder: &'static RepoHolder, file: String, mut src: impl Read, pipe: os_pipe::PipeWriter) -> anyhow::Result<()> {
    let repo = repo_holder.get_without_refresh();
    let package = repo.packages.get(file.as_str()).ok_or_else(|| anyhow!("Package is None"))?;

    let (tx, rx) = mpsc::channel();
    let mut dst = PartialCacheWriter::new(rx);

    dst.add_pipe(pipe);
    package.cache.set(DataSource::Partial(tx));

    let mut do_transfer = || -> std::io::Result<()> {
        let mut buffer = [0u8; 16384];
        while let len@1.. = src.read(&mut buffer)? {
            dst.write_all(&buffer[..len]);
        }
        Ok(())
    };
    if let Err(err) = do_transfer() {
        package.cache.set(DataSource::Empty);
        return Err(err.into());
    }
    if Sha256::digest(dst.data()).as_slice() != package.desc.sha256sum {
        package.cache.set(DataSource::Empty);
        bail!("Checksums do not match");
    }
    dst.release_data_and_flush(|data| {
        package.cache.set(DataSource::Memory(data));
    });
    Ok(())
}

pub fn get_package(repo_holder: &'static RepoHolder, file: &str) -> anyhow::Result<Response> {
    let repo = repo_holder.get_or_refresh();
    let Some(package) = repo.packages.get(file) else {
        return Ok(Response::empty_404());
    };
    let response_body = match package.cache.get() {
        DataSource::Empty => {
            let (reader, writer) = os_pipe::pipe()?;
            let mut mirrors = package.mirrors.clone();
            mirrors.shuffle(&mut rand::rng());

            for mirror in mirrors {
                let url = PathBuf::from(mirror.get(&repo_holder.name)).join(file);
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
        DataSource::Partial(tx) => {
            let (reader, writer) = os_pipe::pipe()?;
            tx.send(writer)?;
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

