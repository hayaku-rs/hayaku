#![feature(proc_macro)]

extern crate cookie;
extern crate hayaku_http;
extern crate hayaku_path;
extern crate marksman_escape;

pub use cookie::Cookie;
pub use hayaku_http::{Header, Http, Request, Response, Method, Status};
pub use hayaku_path::{Router, get_path_params};
use marksman_escape::Escape;

pub fn escape_html(input: &str) -> String {
    let escaped = Escape::new(input.bytes()).collect();
    String::from_utf8(escaped).unwrap()
}
