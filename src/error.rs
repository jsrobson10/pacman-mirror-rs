use std::error::Error;


pub fn handle_fallible<E>(res: Result<rouille::Response, E>) -> rouille::Response where E: Error {
	match res {
		Ok(res) => res,
		Err(err) => {
			eprintln!("Error: {err}");
			rouille::Response::html(maud::html! {
				h1 { "Error 500" }
			})
		}
	}
}

