use rouille::Response;


pub fn check(res: anyhow::Result<Response>) -> Response {
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

