#![feature(proc_macro)]

extern crate cookie;
extern crate hayaku_http;
extern crate hayaku_path;
extern crate marksman_escape;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashMap;

pub use cookie::Cookie;
pub use hayaku_http::{Http, Request, ResponseWriter, Method, Status};
pub use hayaku_path::Router;
use marksman_escape::Escape;

pub fn get_path_params(req: &Request) -> HashMap<String, String> {
    serde_json::from_slice(&*req.user_data.borrow()).unwrap()
}

pub fn escape_html(input: &str) -> String {
    let escaped = Escape::new(input.bytes()).collect();
    String::from_utf8(escaped).unwrap()
}
