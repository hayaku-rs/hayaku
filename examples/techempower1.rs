extern crate hayaku;
extern crate regex;
extern crate rustc_serialize;

use hayaku::{Http, Request, ResponseWriter};
use regex::Regex;

use std::io::Write;
use std::rc::Rc;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let mut http = Http::new(());
    http.handle_func(Regex::new(r"/json").unwrap(), Rc::new(json_handler));
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
