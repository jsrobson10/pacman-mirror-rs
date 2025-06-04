use maud::html;
use rouille::{Request, Response};


pub fn get(_: &Request) -> Response {
	Response::html(html! {
		(maud::DOCTYPE)
		head {
			title { "Index of /" }
			style { (maud::PreEscaped("
			
			th, td {
				padding: 0 1.5em;
			}
			table {
				border: 1px black solid;
			}

			")) }
		}
		body {
			h1 { "Index of /" }
			table {
				tr {
					th { "Filename" }
					th { "State" }
					th { "Mirror" }
				}
			}
		}
	})
}

