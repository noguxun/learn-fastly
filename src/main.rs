use anyhow::Result;
use fastly::http::{header, Method, StatusCode};
use fastly::{mime, Error, Request, Response};

const BACKEND_NAME: &str = "backend_name";

#[fastly::main]
fn main(mut req: Request) -> Result<Response, Error> {
    log_setting(3);

    // Make any desired changes to the client request.
    req.set_header(header::HOST, "example.com");

    // Filter request methods...
    match req.get_method() {
        // Allow GET and HEAD requests.
        &Method::GET | &Method::HEAD => (),

        // Accept PURGE requests; it does not matter to which backend they are sent.
        m if m == "PURGE" => return Ok(req.send(BACKEND_NAME)?),

        // Deny anything else.
        _ => {
            return Ok(Response::from_status(StatusCode::METHOD_NOT_ALLOWED)
                .with_header(header::ALLOW, "GET, HEAD")
                .with_body_text_plain("This method is not allowed\n"))
        }
    };

    // Pattern match on the path.
    match req.get_path() {
        // If request is to the `/` path, send a default response.
        "/" => Ok(Response::from_status(StatusCode::OK)
            .with_content_type(mime::TEXT_HTML_UTF_8)
            .with_body("<iframe src='https://developer.fastly.com/compute-welcome' style='border:0; position: absolute; top: 0; left: 0; width: 100%; height: 100%'></iframe>\n")),

        // If request is to the `/backend` path, send to a named backend.
        "/backend" => {
            // Request handling logic could go here...  E.g., send the request to an origin backend
            // and then cache the response for one minute.
            req.set_ttl(60);
            Ok(req.send(BACKEND_NAME)?)
        }

        // Catch all other requests and return a 404.
        _ => {
            Ok(Response::from_status(StatusCode::NOT_FOUND)
            .with_body_text_plain("The page you requested could not be found\n"))
        }
    }
}

mod my_log_module {
    pub fn do_a_thing() {
        log::warn!("This won't be written, because this module's max level is Error...");
        log::error!("...but this will be written");
    }
}

fn log_setting(log_setup: u8) {
    if log_setup == 0 {
        log_fastly::Logger::builder()
            .max_level(log::LevelFilter::Warn)
            .default_endpoint("my_log0")
            .echo_stdout(true)
            .echo_stderr(true)
            .init();

        // this log will also be echoed to stdout and stderr
        log::warn!("hello world!");
    } else if log_setup == 1 {
        log_fastly::Logger::builder()
            .max_level(log::LevelFilter::Warn)
            .default_endpoint("my_log_default")
            .endpoint("my_log2")
            .endpoint_level("my_log1", log::LevelFilter::Debug)
            .init();

        log::warn!(target: "my_log1", "log with target");
        log::warn!(target: "my_log2", "log with target");

        // without target, log goes to default log point
        log::warn!("log to default");
    } else if log_setup == 2 {
        log_fastly::Logger::builder()
            .max_level(log::LevelFilter::Warn)
            .default_endpoint("my_log_default")
            .filter_module("my_.*_module", log::LevelFilter::Error)
            .init();

        log::warn!("this will not be printed, not matching my_.*_module");
        my_log_module::do_a_thing();
    } else {
        log_fastly::init_simple("my_log_default", log::LevelFilter::Info);
    }

    // expecting an Err here for second init
    assert!(!log_fastly::Logger::builder().try_init().is_ok());

    fastly::log::set_panic_endpoint("my_log_default").unwrap();
}
