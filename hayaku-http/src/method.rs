use std::convert::From;
use minihttp::enums::headers::Method as mhttpMethod;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Method {
    Options,
    Get,
    Head,
    Post,
    Put,
    Patch,
    Delete,
    Trace,
    Connect,
    Other(String),
}

impl From<mhttpMethod> for Method {
    fn from(method: mhttpMethod) -> Method {
        use Method::*;
        match method {
            mhttpMethod::Options => Options,
            mhttpMethod::Get => Get,
            mhttpMethod::Head => Head,
            mhttpMethod::Post => Post,
            mhttpMethod::Put => Put,
            mhttpMethod::Patch => Patch,
            mhttpMethod::Delete => Delete,
            mhttpMethod::Trace => Trace,
            mhttpMethod::Connect => Connect,
            mhttpMethod::Other(s) => Other(s),
        }
    }
}

impl<'a> From<&'a mhttpMethod> for Method {
    fn from(method: &'a mhttpMethod) -> Method {
        use Method::*;
        match *method {
            mhttpMethod::Options => Options,
            mhttpMethod::Get => Get,
            mhttpMethod::Head => Head,
            mhttpMethod::Post => Post,
            mhttpMethod::Put => Put,
            mhttpMethod::Patch => Patch,
            mhttpMethod::Delete => Delete,
            mhttpMethod::Trace => Trace,
            mhttpMethod::Connect => Connect,
            mhttpMethod::Other(ref s) => Other(s.clone()),
        }
    }
}

impl<'a> From<&'a str> for Method {
    fn from(s: &'a str) -> Method {
        match s {
            "OPTIONS" => Method::Options,
            "GET" => Method::Get,
            "HEAD" => Method::Head,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "PATCH" => Method::Patch,
            "DELETE" => Method::Delete,
            "TRACE" => Method::Trace,
            "CONNECT" => Method::Connect,
            s => Method::Other(s.to_string()),
        }
    }
}
