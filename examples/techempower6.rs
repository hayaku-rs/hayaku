extern crate hayaku;
extern crate regex;

use hayaku::{Http, Request, ResponseWriter, Status};
use regex::Regex;

use std::rc::Rc;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let mut http = Http::new(());
    http.handle_func(Regex::new(r"/plaintext").unwrap(), Rc::new(plaintext_handler));
    http.listen_and_serve(addr);
}

fn plaintext_handler(_req: &Request, res: &mut ResponseWriter, _ctx: &()) {
    let data = b"Hello, World!";

    res.status(Status::Ok);
    res.add_header("Content-Type", "text/plain").unwrap();
    res.add_length(data.len() as u64).unwrap();
    if res.done_headers().unwrap() {
        res.write_body(data);
    }
}
