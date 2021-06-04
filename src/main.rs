mod log;
mod req;
mod stream;

use crate::log::*;
use crate::req::*;
use crate::stream::*;
use anyhow::Result;
use fastly::http::{header, Method, StatusCode};
use fastly::{mime, Backend, Error, Request, Response};

static BACKEND_NAME: &str = "httpbin";

// We're not using `#[fastly::main]`
fn main() -> Result<(), Error> {
    log_setting(LogSetting::Simple);
    let mut req = Request::from_client();

    // Make any desired changes to the client request.
    req.set_header(header::HOST, "httpbin.org");

    // Filter request methods...
    match req.get_method() {
        // Allow GET and HEAD requests.
        &Method::GET | &Method::HEAD | &Method::POST => (),

        // Accept PURGE requests; it does not matter to which backend they are sent.
        m if m == "PURGE" => {
            req.send(BACKEND_NAME)?.send_to_client();
            return Ok(());
        }

        // Deny anything else.
        _ => {
            Response::from_status(StatusCode::METHOD_NOT_ALLOWED)
                .with_header(header::ALLOW, "GET, HEAD")
                .with_body_text_plain("This method is not allowed\n")
                .send_to_client();

            return Ok(());
        }
    };

    // Pattern match on the path.
    match req.get_path() {
        // If request is to the `/` path, send a default response.
        "/" => {
            Response::from_status(StatusCode::OK)
                .with_content_type(mime::TEXT_HTML_UTF_8)
                .with_body("<iframe src='https://developer.fastly.com/compute-welcome' style='border:0; position: absolute; top: 0; left: 0; width: 100%; height: 100%'></iframe>\n")
                .send_to_client();

            Ok(())
        }

        // If request is to the `/backend` path, send to a named backend.
        "/backend0" => {
            // Request handling logic could go here...  E.g., send the request to an origin backend
            // and then cache the response for one minute.
            req.set_ttl(60);
            req.send(BACKEND_NAME)?.send_to_client();

            Ok(())
        }

        "/backend1" => {
            let be = Backend::from_name(BACKEND_NAME).unwrap();
            println!("backend info {}", be.name());

            let backend_string = be.into_string();
            println!("backend into_string {}", &backend_string);

            let be_another = Backend::from_name(&backend_string).unwrap();

            req.send(be_another)?.send_to_client();

            Ok(())
        }

        "/stream0" => stream_zeros_to_client(),

        "/stream1" => stream_origin_to_client(BACKEND_NAME, true),

        "/stream2" => stream_origin_to_client(BACKEND_NAME, false),

        path if path.starts_with("/req") => req_process(req, BACKEND_NAME),

        _ => {
            Response::from_status(StatusCode::NOT_FOUND)
                .with_body_text_plain("The page you requested could not be found\n")
                .send_to_client();
            Ok(())
        }
    }
}
