use std::{io::Cursor, path::PathBuf, sync::{mpsc, Arc}, time::SystemTime};
use log::error;
use rouille::{Response, ResponseBody};
use tar::{EntryType, Header};

use crate::{database::Repo, Index};

impl Index {
    fn send_database(&self, writer: os_pipe::PipeWriter, repo: Arc<Repo>) -> anyhow::Result<()> {
        let mut tar_builder = tar::Builder::new(flate2::write::GzEncoder::new(writer, flate2::Compression::new(1)));

        if repo.should_refresh() {
            let (tx, rx) = mpsc::channel::<()>();
            std::thread::spawn({
                let repo = repo.clone();
                move || repo.refresh_if_ready(Some(tx))
            });
            // wait for the signal (its result doesn't matter)
            _ = rx.recv();
        }
        repo.get_from_mirrors(|desc| {
            let path = PathBuf::from(format!("{}-{}", desc.name.as_ref(), desc.version.as_ref()));
            let now = SystemTime::UNIX_EPOCH.elapsed().map(|v| v.as_secs()).unwrap_or(0);
    
            let send_file = |builder: &mut tar::Builder<_>, name: &str, bytes: &[u8]| -> anyhow::Result<()> {
                builder.append(&{
                    let mut v = Header::new_gnu();
                    v.set_path(path.join(name))?;
                    v.set_entry_type(EntryType::file());
                    v.set_size(bytes.len().try_into()?);
                    v.set_mode(0o644);
                    v.set_mtime(now);
                    v.set_cksum();
                    v
                }, Cursor::new(bytes))?;
                Ok(())
            };
            tar_builder.append(&{
                let mut v = Header::new_gnu();
                v.set_path(&path)?;
                v.set_entry_type(EntryType::dir());
                v.set_mode(0o755);
                v.set_mtime(now);
                v.set_cksum();
                v
            }, std::io::empty())?;
    
            send_file(&mut tar_builder, "desc", &desc.write_to_vec()?)?;
    
            Ok(())
        })
    }
    pub fn get_database(self: &Arc<Self>, repo: Arc<Repo>) -> anyhow::Result<Response> {
        let (reader, writer) = os_pipe::pipe()?;
        let db = self.clone();

        std::thread::spawn(move || {
            if let Err(err) = db.send_database(writer, repo) {
                error!("{err}");
            }
        });
    
        Ok(Response {
            status_code: 200,
            headers: vec![
                ("Content-Type".into(), "application/x-tar".into()),
                ("Content-Encoding".into(), "x-gzip".into()),
            ],
            data: ResponseBody::from_reader(Box::new(reader)),
            upgrade: None,
        })
    }
}

