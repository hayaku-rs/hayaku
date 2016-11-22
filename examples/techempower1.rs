extern crate hayaku;
extern crate rustc_serialize;

use hayaku::{Http, Router, Request, ResponseWriter};

use std::io::Write;
use std::rc::Rc;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();

    let mut router = Router::new();
    router.get("/json", Rc::new(json_handler)).unwrap();

    let http = Http::new(router, ());
    http.listen_and_serve(addr);
}

#[derive(RustcEncodable)]
struct Message {
    message: String,
}

fn json_handler(_req: &Request, res: &mut ResponseWriter, _ctx: &()) {
    let msg = Message { message: "Hello, World!".to_string() };
    let data = &rustc_serialize::json::encode(&msg).unwrap();

    res.add_header("Content-Type", "application/json");
    res.write_all(data.as_bytes()).unwrap();
}
