use std::io::Cursor;
use rouille::{Response, ResponseBody};

use crate::database::{package::DataSource, repo::holder::RepoHolder, DB};



pub fn get_package(repo: &'static RepoHolder, file: &str) -> anyhow::Result<Response> {
	let repo = repo.get_or_refresh();
	let Some(package) = repo.packages.get(file) else {
		return Ok(Response::empty_404());
	};

	let data_src = package.source.read().unwrap();
	//let (reader, writer) = os_pipe::pipe()?;
	let response_body: ResponseBody;

	match &*data_src {
		DataSource::None => {
			todo!()
		}
		DataSource::Partial(reader) => {
			let reader = reader.reader();
			response_body = match reader.get_size_hint() {
				Some(len) => ResponseBody::from_reader_and_size(reader, len),
				None => ResponseBody::from_reader(reader),
			};
		}
		DataSource::Memory(buff) => {
			response_body = ResponseBody::from_reader_and_size(Cursor::new(buff.clone()), buff.len());
		}
	}

	Ok(Response {
		status_code: 200,
		headers: vec![
			("Content-Type".into(), "application/x-tar".into()),
		],
		data: response_body,
		upgrade: None,
	})
}

