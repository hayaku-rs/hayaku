#![feature(proc_macro)]

#[macro_use]
extern crate log;
extern crate hayaku_http;
#[macro_use(quick_error)]
extern crate quick_error;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod error;
mod trie;

use hayaku_http::{Handler, Method, Request, RequestHandler, ResponseWriter, Status};

use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;

pub use error::Error;

use trie::TrieNode;

type Tree<T> = HashMap<Method, TrieNode<Rc<RequestHandler<T>>>>;

#[derive(Clone)]
pub struct Router<T: Clone> {
    trees: Tree<T>,
    /// Enables automatic redirection if the current route can't be matched but
    /// a handler for the path with (without) the trailing slash exists.
    /// For example if /foo/ is requested but a route only exists for /foo, the
    /// client is redirected to /foo with http status code 301 for GET requests
    /// and 307 for all other request methods.
    pub redirect_trailing_slash: bool,
    /// If enabled, the router tries to fix the current request path, if no
    /// handle is registered for it.
    /// First superfluous path elements like ../ or // are removed.
    /// Afterwards the router does a case-insensitive lookup of the cleaned path.
    /// If a handle can be found for this route, the router makes a redirection
    /// to the corrected path with status code 301 for GET requests and 307 for
    /// all other request methods.
    /// For example /FOO and /..//Foo could be redirected to /foo.
    /// `redirect_trailing_slash` is independent of this option.
    pub redirect_fixed_path: bool,
    /// If enabled, the router checks if another method is allowed for the
    /// current route, if the current request can not be routed.
    /// If this is the case, the request is answered with `Method Not Allowed`
    /// and HTTP status code 405.
    /// If no other Method is allowed, the request is delegated to the NotFound
    /// handler.
    pub handle_method_not_allowed: bool,
    /// If enabled, the router automatically replies to OPTIONS requests.
    /// Custom OPTIONS handlers take priority over automatic replies.
    pub handle_options: bool,
    /// Configurable handler which is called when no matching route is
    /// found. If it is `None`, the default 404 handler is used.
    not_found: Option<Rc<RequestHandler<T>>>,
}

impl<T: Clone> Router<T> {
    pub fn new() -> Self {
        Router {
            trees: HashMap::new(),
            redirect_trailing_slash: true,
            redirect_fixed_path: true,
            handle_method_not_allowed: true,
            handle_options: true,
            not_found: None,
        }
    }

    pub fn set_not_found_handler(&mut self, handler: Rc<RequestHandler<T>>) {
        self.not_found = Some(handler);
    }

    /// `get` is a shortcut for `Self::handle(Method::Get, path, handle)`.
    pub fn get<S: Into<String>>(&mut self,
                                path: S,
                                handle: Rc<RequestHandler<T>>)
                                -> Result<(), Error> {
        self.handle(Method::Get, path, handle)
    }

    /// `head` is a shortcut for `Self::handle(Method::Head, path, handle)`.
    pub fn head<S: Into<String>>(&mut self,
                                 path: S,
                                 handle: Rc<RequestHandler<T>>)
                                 -> Result<(), Error> {
        self.handle(Method::Head, path, handle)
    }

    /// `options` is a shortcut for `Self::handle(Method::Options, path, handle)`.
    pub fn options<S: Into<String>>(&mut self,
                                    path: S,
                                    handle: Rc<RequestHandler<T>>)
                                    -> Result<(), Error> {
        self.handle(Method::Options, path, handle)
    }

    /// `post` is a shortcut for `Self::handle(Method::Post, path, handle)`.
    pub fn post<S: Into<String>>(&mut self,
                                 path: S,
                                 handle: Rc<RequestHandler<T>>)
                                 -> Result<(), Error> {
        self.handle(Method::Post, path, handle)
    }

    /// `put` is a shortcut for `Self::handle(Method::Put, path, handle)`.
    pub fn put<S: Into<String>>(&mut self,
                                path: S,
                                handle: Rc<RequestHandler<T>>)
                                -> Result<(), Error> {
        self.handle(Method::Put, path, handle)
    }

    /// `patch` is a shortcut for `Self::handle(Method::Patch, path, handle)`.
    pub fn patch<S: Into<String>>(&mut self,
                                  path: S,
                                  handle: Rc<RequestHandler<T>>)
                                  -> Result<(), Error> {
        self.handle(Method::Patch, path, handle)
    }

    /// `delete` is a shortcut for `Self::handle(Method::Delete, path, handle)`.
    pub fn delete<S: Into<String>>(&mut self,
                                   path: S,
                                   handle: Rc<RequestHandler<T>>)
                                   -> Result<(), Error> {
        self.handle(Method::Delete, path, handle)
    }

    /// Registers a new request handle with the given path and method.
    ///
    /// For GET, POST, PUT, PATCH, and DELETE requests the respective
    /// shortcut functions can be used.
    ///
    /// This function is intended for bulk loading and to allow the usage
    /// of less frequently used, non-standardized or custom methods
    /// (e.g. for internal communication with a proxy).
    pub fn handle<S: Into<String>>(&mut self,
                                   method: Method,
                                   path: S,
                                   handle: Rc<RequestHandler<T>>)
                                   -> Result<(), Error> {
        let path = path.into();
        if !path.starts_with('/') {
            return Err(Error::StartWithSlash(path.to_string()));
        }

        if self.trees.get(&method).is_none() {
            let root = TrieNode::new();
            self.trees.insert(method.clone(), root);
        }

        // TODO(nokaa): we should probably not unwrap here.
        // It may be possible for the retrieval to fail, even
        // though we check just before this.
        let mut root = self.trees.get_mut(&method).unwrap();
        root.insert(path, handle);

        Ok(())
    }
}

impl<T: Clone> Handler<T> for Router<T> {
    // Handler makes the router implement the fasthttp.ListenAndServe interface.
    fn handler(&self, req: &Request, res: &mut ResponseWriter, ctx: &T) {
        let path = req.path;
        debug!("path: {}", path);
        let method = &req.method;
        debug!("method: {:?}", method);
        if let Some(root) = self.trees.get(method) {
            match root.get(path) {
                Some((val, map)) => {
                    let serialized = serde_json::to_vec(&map).unwrap();
                    *req.user_data.borrow_mut() = serialized;
                    val.unwrap()(req, res, ctx);
                }
                None => {
                    if self.not_found.is_none() {
                        // Default handler
                        res.status(Status::NotFound);
                        let msg = String::from("404, path \"") + path + "\" not found :(";
                        res.write_all(msg.as_bytes()).unwrap();
                    } else {
                        // We have already checked that self.not_found is not
                        // `None`, so unwrapping should be okay.
                        let handle = self.not_found.clone().unwrap();
                        handle(req, res, ctx);
                    }
                }
            }
        }
    }
}
