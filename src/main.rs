use rouille::router;

pub mod index;
pub mod error;

fn main() {
	rouille::start_server("localhost:8080", |req| {
		router!(req,
			(GET) (/) => { index::get(req) },
			_ => rouille::Response::empty_404()
		)
	});
}

