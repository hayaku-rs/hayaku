#![feature(proc_macro)]

extern crate hayaku_http;
extern crate hayaku_path;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashMap;

pub use hayaku_http::{Http, Request, ResponseWriter, Method, Status};
pub use hayaku_path::Router;

pub fn get_path_params(req: &Request) -> HashMap<String, String> {
    serde_json::from_slice(&*req.user_data.borrow()).unwrap()
}
