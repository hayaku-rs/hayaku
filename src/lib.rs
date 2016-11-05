/* Copyright (C)  2016 nokaa <nokaa@cock.li>
 * This software is licensed under the terms of the
 * GNU Affero General Public License. You should have
 * received a copy of this license with this software.
 * The license may also be found at https://gnu.org/licenses/agpl.txt
 * */

#[macro_use]
extern crate log;
extern crate futures;
extern crate tokio_core;
extern crate tokio_service;
extern crate tk_bufstream;
extern crate minihttp;
extern crate regex;
extern crate urlencoded;
extern crate multipart;

mod path;
mod request;
mod response;

pub use path::Path;
pub use response::ResponseWriter;
pub use request::Request;
pub use minihttp::Status;

use futures::{Async, Finished, finished};
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tokio_service::Service;
use tk_bufstream::IoBuf;
use minihttp::{Error, ResponseFn};

use std::io::Write;
use std::net::SocketAddr;
use std::rc::Rc;

type Response = ResponseFn<Finished<IoBuf<TcpStream>, Error>, TcpStream>;

#[derive(Clone)]
pub struct Http<T: Clone> {
    routes: Vec<Path>,
    route_handlers: Vec<Rc<Fn(&Request, &mut ResponseWriter, &T)>>,
    not_found: Option<Rc<Fn(&Request, &mut ResponseWriter, &T)>>,
    context: T,
}

impl<T: 'static + Clone> Service for Http<T> {
    type Request = minihttp::Request;
    type Response = Response;
    type Error = Error;
    type Future = Finished<Self::Response, Error>;

    fn call(&self, req: minihttp::Request) -> Self::Future {
        // Retrieve the function associated with this path
        let index = self.match_route(&req.path);
        let func = match index {
            Some(i) => self.route_handlers[i].clone(),
            None => {
                match &self.not_found {
                    &Some(ref f) => f.clone(),
                    &None => {
                        return finished(ResponseFn::new(move |res| {
                            let mut res = ResponseWriter::new(res);
                            res.status(Status::NotFound);
                            if let Err(e) = res.write_all(b"404 - Page not found") {
                                error!("{}", e);
                            }
                            res.done()
                        }));
                    }
                }
            }
        };
        let context = self.context.clone();

        // Note: rather than allocating a response object, we return
        // a lambda that pushes headers into `ResponseWriter` which
        // writes them directly into response buffer without allocating
        // intermediate structures
        finished(ResponseFn::new(move |res| {
            let mut res = ResponseWriter::new(res);
            let req = Request::from(&req);
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
        Http {
            routes: Vec::new(),
            route_handlers: Vec::new(),
            not_found: None,
            context: context,
        }
    }

    /// Add a function to handle the given `path`.
    pub fn handle_func(&mut self, expr: Path, func: Rc<Fn(&Request, &mut ResponseWriter, &T)>) {
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
        // The (size, index) of the best match
        let mut best_match = (0, None);

        let mut index = 0;
        for expr in &self.routes {
            match expr {
                &Path::Exact(ref s) => {
                    if s == route {
                        info!("best match: {}", s);
                        return Some(index);
                    }
                }
                &Path::Regex(ref r) => {
                    if let Some((a, b)) = r.find(route) {
                        if b - a > best_match.0 {
                            info!("best match: {}", r);
                            best_match.0 = b - a;
                            best_match.1 = Some(index);
                        }
                    }
                }
            }
            index += 1;
        }

        best_match.1
    }
}
