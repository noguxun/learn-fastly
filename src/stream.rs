use anyhow::Result;
use fastly::http::{header, StatusCode};
use fastly::{Request, Response};
use std::io::{BufRead, Write};

pub fn stream_origin_to_client(backend_name: &str, unset_origin_cl: bool) -> Result<()> {
    let mut backend_resp = Request::get("https://dummy/html").send(backend_name)?;
    // Take the body so we can iterate through its lines later
    let backend_resp_body = backend_resp.take_body();
    // Start sending the backend response to the client with a now-empty body
    if unset_origin_cl {
        backend_resp.remove_header(header::CONTENT_LENGTH);
    }
    let mut client_body = backend_resp.stream_to_client();

    let mut num_lines = 0;
    for line in backend_resp_body.lines() {
        let line = line.unwrap();
        num_lines += 1;
        // Write the line to the streaming client body
        client_body.write_str(&line);
    }
    client_body.flush()?;

    // Drop the streaming body to allow the client connection to close
    drop(client_body);

    println!("backend response body contained {} lines", num_lines);

    Ok(())
}

pub fn stream_zeros_to_client() -> Result<()> {
    let mut client_body = Response::from_status(StatusCode::OK).stream_to_client();

    for _ in 0..100 {
        client_body.write_str("0");
    }

    client_body.flush()?;

    for _ in 0..100 {
        client_body.write_str("1");
    }

    client_body.flush()?;

    // Drop the streaming body to allow the client connection to close
    drop(client_body);

    Ok(())
}
