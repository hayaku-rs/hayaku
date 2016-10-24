/* Copyright (C)  2016 nokaa <nokaa@cock.li>
 * This software is licensed under the terms of the
 * GNU Affero General Public License. You should have
 * received a copy of this license with this software.
 * The license may also be found at https://gnu.org/licenses/agpl.txt
 * */

#[macro_use]
extern crate chomp;
extern crate futures;
extern crate tokio_core;
extern crate tokio_service;
extern crate tk_bufstream;
extern crate minihttp;
extern crate regex;

pub mod file;
pub mod forms;
pub mod util;

use futures::{Async, Finished, finished};
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tokio_service::Service;
use tk_bufstream::IoBuf;
use minihttp::{Error, ResponseFn};
use regex::Regex;

use std::net::SocketAddr;
use std::rc::Rc;

pub use minihttp::Request;

type Response = ResponseFn<Finished<IoBuf<TcpStream>, Error>, TcpStream>;

pub type ResponseWriter = minihttp::ResponseWriter<TcpStream>;

#[derive(Clone)]
pub struct Http<T: Clone> {
    routes: Vec<Regex>,
    route_handlers: Vec<Rc<Fn(&Request, &mut ResponseWriter, &T)>>,
    not_found: Option<Rc<Fn(&Request, &mut ResponseWriter)>>,
    context: T,
}

impl<T: 'static + Clone> Service for Http<T> {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Future = Finished<Self::Response, Error>;

    fn call(&self, req: Request) -> Self::Future {
        // Retrieve the function associated with this path
        // let path = req.path;
        let index = self.match_route(&req.path);
        let func = match index {
            Some(i) => self.route_handlers[i].clone(),
            None => Rc::new(Http::four_o_four),
        };
        let context = self.context.clone();
        // let func = self.routes.get(&req.path).unwrap().clone();

        // Note: rather than allocating a response object, we return
        // a lambda that pushes headers into `ResponseWriter` which
        // writes them directly into response buffer without allocating
        // intermediate structures
        finished(ResponseFn::new(move |mut res| {
            // Run the function
            func(&req, &mut res, &context);
            // Return the future associated with finishing handling this request
            res.done()
        }))
    }

    fn poll_ready(&self) -> Async<()> {
        Async::Ready(())
    }
}

impl<T: 'static + Clone> Http<T> {
    /// Create a new Http handler
    pub fn new(context: T) -> Http<T> {
        // let func = Rc::new(Http::not_found);
        Http {
            routes: Vec::new(),
            route_handlers: Vec::new(),
            not_found: None,
            context: context,
        }
    }

    /// Add a function to handle the given `path`.
    pub fn handle_func(&mut self, expr: Regex, func: Rc<Fn(&Request, &mut ResponseWriter, &T)>) {
        // self.routes.insert(path, func);
        self.routes.push(expr);
        self.route_handlers.push(func);
        assert_eq!(self.routes.len(), self.route_handlers.len());
    }

    /// Run the server
    pub fn listen_and_serve(self, addr: SocketAddr) {
        let mut lp = Core::new().unwrap();
        minihttp::serve(&lp.handle(), addr, self);
        lp.run(futures::empty::<(), ()>()).unwrap()
    }

    fn match_route(&self, route: &str) -> Option<usize> {
        let mut index = 0;
        for expr in &self.routes {
            if expr.is_match(route) {
                return Some(index);
            }
            index += 1;
        }

        // No matching routes were found
        // In this case we probably want to respond with a 404
        None
    }

    // TODO(nokaa): Serve an actual 404
    fn four_o_four(_req: &Request, res: &mut ResponseWriter, _context: &T) {
        res.status(200, "OK");
        res.add_chunked().unwrap();
        if res.done_headers().unwrap() {
            res.write_body(b"404 - Not found");
        }
    }
}
