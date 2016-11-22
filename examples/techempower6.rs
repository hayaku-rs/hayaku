extern crate hayaku;

use hayaku::{Http, Router, Request, ResponseWriter};

use std::io::Write;
use std::rc::Rc;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();

    let mut router = Router::new();
    router.get("/plaintext", Rc::new(plaintext_handler)).unwrap();

    let http = Http::new(router, ());
    http.listen_and_serve(addr);
}

fn plaintext_handler(_req: &Request, res: &mut ResponseWriter, _ctx: &()) {
    let data = b"Hello, World!";

    res.add_header("Content-Type", "text/plain");
    res.write_all(data).unwrap();
}
