extern crate hayaku;
extern crate rustc_serialize;

use hayaku::{Http, Router, Request, Response};

use std::sync::Arc;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();

    let mut router = Router::new();
    router.get("/json", Arc::new(json_handler)).unwrap();

    let http = Http::new(router, ());
    http.listen_and_serve(addr);
}

#[derive(RustcEncodable)]
struct Message {
    message: String,
}

fn json_handler(_req: &Request, res: &mut Response, _ctx: &()) {
    let msg = Message { message: "Hello, World!".to_string() };
    let data = &rustc_serialize::json::encode(&msg).unwrap();

    res.add_header("Content-Type".to_string(), "application/json".to_string());
    res.body(data.as_bytes());
}
