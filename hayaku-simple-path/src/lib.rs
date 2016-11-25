#[macro_use]
extern crate log;
extern crate hayaku_http;

use hayaku_http::{Handler, Request, RequestHandler, ResponseWriter, Status};
use std::io::Write;
use std::rc::Rc;

pub struct Router<T: Clone> {
    paths: Vec<(String, Rc<RequestHandler<T>>)>,
    not_found: Option<Rc<RequestHandler<T>>>,
}

impl<T: Clone> Router<T> {
    pub fn new() -> Self {
        Router {
            paths: Vec::new(),
            not_found: None,
        }
    }

    pub fn set_not_found_handler(&mut self, handle: Rc<RequestHandler<T>>) {
        self.not_found = Some(handle);
    }

    pub fn add_route(&mut self, route: String, handle: Rc<RequestHandler<T>>) {
        self.paths.push((route, handle));
    }
}

impl<T: Clone> Handler<T> for Router<T> {
    fn handler(&self, req: &Request, res: &mut ResponseWriter, ctx: &T) {
        let path = &req.path;
        for &(ref route, ref handle) in &self.paths {
            if *path == route {
                handle(req, res, ctx);
            }
        }

        // TODO(nokaa): Serve 404 page
        if self.not_found.is_some() {
            let handle = self.not_found.clone().unwrap();
            handle(req, res, ctx);
        } else {
            res.status(Status::NotFound);
            let msg = String::from("404, path \"") + path + "\" not found :(";
            res.write_all(msg.as_bytes()).unwrap();

        }
    }
}
