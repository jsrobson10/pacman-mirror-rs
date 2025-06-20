use log::error;
use rouille::Response;


pub fn check(res: anyhow::Result<Response>) -> Response {
    match res {
        Ok(res) => res,
        Err(err) => {
            error!("{err}");
            rouille::Response::html(maud::html! {
                h1 { "Error 500" }
            })
        }
    }
}

