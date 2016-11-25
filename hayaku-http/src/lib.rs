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

mod handler;
mod method;
mod path;
mod request;
mod response;

pub use handler::Handler;
pub use method::Method;
pub use path::Path;
pub use response::ResponseWriter;
pub use request::Request;
pub use minihttp::Status;

use futures::{Finished, finished};
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tokio_service::Service;
use tk_bufstream::IoBuf;
use minihttp::{Error, ResponseFn};

use std::net::SocketAddr;

// TODO(nokaa): We probably want to enforce the Clone trait bound on `T`
// here. We can't do this until https://github.com/rust-lang/rust/issues/21903
// is resovled. This shouldn't be a problem because when we use this type we
// are constraining `T` to be Clone.
pub type RequestHandler<T> = Fn(&Request, &mut ResponseWriter, &T);

type Response = ResponseFn<Finished<IoBuf<TcpStream>, Error>, TcpStream>;

#[derive(Clone)]
pub struct Http<T: Clone, H: Clone + Handler<T>> {
    handler: H,
    context: T,
    sanitize_input: bool,
}

impl<T: 'static + Clone, H: 'static + Clone + Handler<T>> Service for Http<T, H> {
    type Request = minihttp::Request;
    type Response = Response;
    type Error = Error;
    type Future = Finished<Self::Response, Error>;

    fn call(&self, req: minihttp::Request) -> Self::Future {
        // We declare these variables here to satisfy lifetime requirements.
        // Note that as these are both Rc (smart pointers) we can clone them
        // without issue.
        let handler = self.handler.clone();
        let context = self.context.clone();
        let sanitize = self.sanitize_input;

        finished(ResponseFn::new(move |res| {
            let mut res = ResponseWriter::new(res);
            let req = Request::new(&req, sanitize);
            handler.handler(&req, &mut res, &context);
            res.done()
        }))
    }
}

impl<T: 'static + Clone, H: 'static + Clone + Handler<T>> Http<T, H> {
    /// Create a new Http handler
    pub fn new(handler: H, context: T) -> Self {
        Http {
            handler: handler,
            context: context,
            sanitize_input: false,
        }
    }

    /// Calling this method will cause form data to be HTML-escaped
    /// when parsed.
    pub fn sanitize(&mut self) {
        self.sanitize_input = true;
    }

    /// Run the server
    pub fn listen_and_serve(self, addr: SocketAddr) {
        let mut lp = Core::new().unwrap();
        minihttp::serve(&lp.handle(), addr, move || Ok(self.clone()));
        lp.run(futures::empty::<(), ()>()).unwrap()
    }
}
