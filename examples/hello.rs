extern crate http;
extern crate regex;

use http::{Http, Request, ResponseWriter};
use regex::Regex;

use std::rc::Rc;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let mut http = Http::new();
    http.handle_func(Regex::new(r"/").unwrap(), Rc::new(test_func));
    http.listen_and_serve(addr);
}

fn test_func(_req: &Request, res: &mut ResponseWriter) {
    res.status(200, "OK");
    res.add_chunked().unwrap();
    if res.done_headers().unwrap() {
        res.write_body(b"Hello, world!");
    }
}
