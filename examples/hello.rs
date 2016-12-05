extern crate env_logger;
extern crate hayaku;

use hayaku::{Http, Router, Request, Response};

use std::sync::Arc;

fn main() {
    env_logger::init().unwrap();
    let addr = "127.0.0.1:3000".parse().unwrap();

    let mut router = Router::new();
    router.get("/", Arc::new(hello_handler)).unwrap();

    let http = Http::new(router, ());
    http.listen_and_serve(addr);
}

fn hello_handler(_req: &Request, res: &mut Response, _ctx: &()) {
    let data = "Hello, world!";
    res.body(data.as_bytes());
}
