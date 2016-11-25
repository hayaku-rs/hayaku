use minihttp::{self, enums, request};
use urlencoded::{parse_urlencoded, parse_urlencoded_html_escape};

use super::Method;

use std::cell::RefCell;
use std::collections::HashMap;
use std::net::SocketAddr;

// mod multipart;

pub struct Request<'a> {
    pub method: Method,
    pub path: &'a String,
    pub version: &'a enums::Version,
    pub headers: &'a Vec<(enums::Header, String)>,
    pub body: &'a Option<request::Body>,
    pub peer_addr: &'a SocketAddr,
    request: &'a minihttp::Request,
    form: RefCell<Option<HashMap<String, String>>>,
    pub user_data: RefCell<Vec<u8>>,
    sanitize_input: bool,
}

impl<'a> Request<'a> {
    pub fn new(req: &'a minihttp::Request, sanitize: bool) -> Request<'a> {
        Request {
            method: Method::from(&req.method),
            path: &req.path,
            version: &req.version,
            headers: &req.headers,
            body: &req.body,
            peer_addr: &req.peer_addr,
            request: req,
            form: RefCell::new(None),
            user_data: RefCell::new(Vec::new()),
            sanitize_input: sanitize,
        }
    }

    pub fn has_body(&self) -> bool {
        self.request.has_body()
    }

    pub fn host(&self) -> Option<&str> {
        self.request.host()
    }

    pub fn content_type(&self) -> Option<&str> {
        self.request.content_type()
    }

    pub fn content_length(&self) -> Option<u64> {
        self.request.content_length()
    }

    pub fn transfer_encoding(&self) -> Option<&str> {
        self.request.transfer_encoding()
    }

    pub fn form_value<S: Into<String>>(&self, key: S) -> Option<String> {
        let key = key.into();

        if *self.form.borrow() == None {
            match *self.body {
                None => return None,
                Some(ref b) => {
                    let body = &b.data[..];
                    info!("Request body: {:?}", body);
                    let m = if self.sanitize_input {
                        parse_urlencoded_html_escape(body)
                    } else {
                        parse_urlencoded(body)
                    };
                    let m = match m {
                        Ok(m) => m,
                        Err(e) => {
                            // For now if we can't parse the form we
                            // just return an empty map
                            debug!("Error parsing form: {}", e);
                            HashMap::new()
                        }
                    };
                    *self.form.borrow_mut() = Some(m);
                }
            }
        }

        match *self.form.borrow() {
            Some(ref map) => {
                match map.get(&key) {
                    None => None,
                    Some(s) => Some(s.clone()),
                }
            }
            None => unimplemented!(),
        }
    }
}
