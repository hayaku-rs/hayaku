extern crate hayaku;
extern crate regex;

use hayaku::{util, Http, Request, ResponseWriter};
use regex::Regex;

use std::rc::Rc;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let mut http = Http::new(());
    http.handle_func(Regex::new(r"/").unwrap(), Rc::new(hello_handler));
    http.listen_and_serve(addr);
}

fn hello_handler(_req: &Request, res: &mut ResponseWriter, _ctx: &()) {
    util::send_string(res, b"Hello, world!");
}
