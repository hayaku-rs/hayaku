use std::io;

use minihttp::Status;

use file;

use ResponseWriter;

/// Handles redirects. Redirects `res` to `location`
/// with code `code`. `data` should be a file/message that
/// explains what happened to the user.
/// Returns an `Err(String)` if an invalid code is received.
pub fn redirect(res: &mut ResponseWriter,
                data: &[u8],
                location: &[u8],
                code: u16)
                -> Result<(), String> {
    res.status(Status::from(code).unwrap());
    res.add_header("Location", location).unwrap();
    res.add_length(data.len() as u64).unwrap();
    if res.done_headers().unwrap() {
        res.write_body(data);
    }

    Ok(())
}

/// Handles sending a server error. This is useful
/// for things like a 404. `data` should explain what
/// happened to the user.
/// Returns an `Err(String)` if an invalid code is received.
pub fn error(res: &mut ResponseWriter, data: &[u8], code: u16) -> Result<(), String> {
    res.status(Status::from(code).unwrap());
    res.add_length(data.len() as u64).unwrap();
    if res.done_headers().unwrap() {
        res.write_body(data);
    }

    Ok(())
}

/// Sends `data` to the client with status 200.
pub fn send_string(res: &mut ResponseWriter, data: &[u8]) {
    res.status(Status::Ok);
    res.add_length(data.len() as u64).unwrap();
    if res.done_headers().unwrap() {
        res.write_body(data);
    }
}

/// Sends `data` to the client with status 200.
/// Sets `Content-Type` header to `text/plain`.
pub fn send_string_raw(res: &mut ResponseWriter, data: &[u8]) {
    res.status(Status::Ok);
    // Add `Content-Type` header to ensure data is interpreted
    // as plaintext
    res.add_header("Content-Type", "text/plain; charset=utf-8".as_bytes())
        .unwrap();
    res.add_length(data.len() as u64).unwrap();
    if res.done_headers().unwrap() {
        res.write_body(data);
    }
}

/// Sends data read from `filename` to the client
/// with status 200.
///
/// Returns `Err(io::Error)` if `filename` cannot be read.
pub fn send_file(res: &mut ResponseWriter, filename: &str) -> Result<(), io::Error> {
    let data = &try!(file::read_file(filename))[..];

    res.status(Status::Ok);
    res.add_length(data.len() as u64).unwrap();
    if res.done_headers().unwrap() {
        res.write_body(data);
    }

    Ok(())
}
