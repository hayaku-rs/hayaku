extern crate hayaku;
extern crate regex;

use hayaku::{Http, Path, Request, ResponseWriter};
use regex::Regex;

use std::io::Write;
use std::rc::Rc;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let mut http = Http::new(());
    http.handle_func(Path::from(String::from("/plaintext")),
                     Rc::new(plaintext_handler));
    http.listen_and_serve(addr);
}

fn plaintext_handler(_req: &Request, res: &mut ResponseWriter, _ctx: &()) {
    let data = b"Hello, World!";

    res.add_header("Content-Type", "text/plain");
    res.write_all(data).unwrap();
}
