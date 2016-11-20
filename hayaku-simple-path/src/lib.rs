#[macro_use]
extern crate log;
extern crate hayaku_http;

use hayaku_http::{Handler, Request, RequestHandler, ResponseWriter};
use std::rc::Rc;

pub struct Router<T: Clone> {
    paths: Vec<(String, Rc<RequestHandler<T>>)>,
}

impl<T: Clone> Router<T> {
    pub fn new() -> Self {
        Router { paths: Vec::new() }
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
    }
}
