use std::{io::Cursor, path::Path, time::SystemTime};
use rouille::{Response, ResponseBody};
use tar::{EntryType, Header};

use crate::database::{desc, repo::holder::RepoHolder};

fn send_database(writer: os_pipe::PipeWriter, repo: &RepoHolder, files: bool) -> anyhow::Result<()> {
	let repo = repo.get_or_refresh();
	let mut tar_builder = tar::Builder::new(flate2::write::GzEncoder::new(writer, flate2::Compression::new(1)));

	for package in repo.packages.values() {
		let path = Path::new(package.name.as_ref());
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
			v.set_path(path)?;
			v.set_entry_type(EntryType::dir());
			v.set_mode(0o755);
			v.set_mtime(now);
			v.set_cksum();
			v
		}, std::io::empty())?;
		if let Some(desc) = package.desc.as_ref() {
			send_file(&mut tar_builder, "desc", desc.as_bytes())?;
		}
		if let Some(files) = package.files.as_ref().filter(|_| files) {
			send_file(&mut tar_builder, "files", files.as_bytes())?;
		}
	}

	Ok(())
}

pub fn get_database(repo: &'static RepoHolder, files: bool) -> anyhow::Result<Response> {
	let (reader, writer) = os_pipe::pipe()?;

	rayon::spawn(move || {
		if let Err(err) = send_database(writer, repo, files) {
			eprintln!("Error in stream: {err}");
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

