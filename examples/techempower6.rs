extern crate hayaku;

use hayaku::{Http, Router, Request, Response};

use std::sync::Arc;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();

    let mut router = Router::new();
    router.get("/plaintext", Arc::new(plaintext_handler)).unwrap();

    Http::new(router, ()).listen_and_serve(addr);
}

fn plaintext_handler(_req: &Request, res: &mut Response, _ctx: &()) {
    res.add_header("Content-Type".to_string(), "text/plain".to_string());
    res.body(b"Hello, World!").unwrap();
}
