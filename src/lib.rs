/* Copyright (C)  2016 nokaa <nokaa@cock.li>
 * This software is licensed under the terms of the
 * GNU Affero General Public License. You should have
 * received a copy of this license with this software.
 * The license may also be found at https://gnu.org/licenses/agpl.txt
 * */

extern crate futures;
extern crate tokio_core;
extern crate tokio_service;
extern crate tk_bufstream;
extern crate minihttp;

use futures::{Async, Finished, finished};
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;
use tokio_service::Service;
use tk_bufstream::IoBuf;
use minihttp::{Error, ResponseFn};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::rc::Rc;

pub use minihttp::{Request, ResponseWriter};

type Response = ResponseFn<Finished<IoBuf<TcpStream>, Error>, TcpStream>;

#[derive(Clone)]
pub struct Http {
    routes: HashMap<String, Rc<Fn(&Request, &mut ResponseWriter<TcpStream>)>>,
}

impl Service for Http {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Future = Finished<Self::Response, Error>;

    fn call(&self, req: Request) -> Self::Future {
        // Retrieve the function associated with this path
        let func = self.routes.get(&req.path).unwrap().clone();

        // Note: rather than allocating a response object, we return
        // a lambda that pushes headers into `ResponseWriter` which
        // writes them directly into response buffer without allocating
        // intermediate structures
        finished(ResponseFn::new(move |mut res| {
            // Run the function
            func(&req, &mut res);
            // Return the future associated with finishing handling this request
            res.done()
        }))
    }

    fn poll_ready(&self) -> Async<()> {
        Async::Ready(())
    }
}

impl Http {
    /// Create a new Http handler
    pub fn new() -> Http {
        Http { routes: HashMap::new() }
    }

    /// Add a function to handle the given `path`.
    pub fn handle_func(&mut self,
                       path: String,
                       func: Rc<Fn(&Request, &mut ResponseWriter<TcpStream>)>) {
        self.routes.insert(path, func);
    }

    /// Run the server
    pub fn listen_and_serve(self, addr: SocketAddr) {
        let mut lp = Core::new().unwrap();
        minihttp::serve(&lp.handle(), addr, self);
        lp.run(futures::empty::<(), ()>()).unwrap()
    }
}
