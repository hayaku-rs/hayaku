extern crate hayaku;
extern crate regex;

use hayaku::{Http, Path, Request, ResponseWriter};

use std::io::Write;
use std::rc::Rc;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let mut http = Http::new(());
    http.handle_func(Path::from(String::from("/")), Rc::new(hello_handler));
    http.listen_and_serve(addr);
}

fn hello_handler(_req: &Request, res: &mut ResponseWriter, _ctx: &()) {
    res.write(b"Hello, world!").unwrap();
}
