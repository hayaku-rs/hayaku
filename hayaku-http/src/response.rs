use futures::Finished;
use tokio_core::net::TcpStream;
use tk_bufstream::{Flushed, IoBuf};
use minihttp::{self, Status};

use std::fmt::Display;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub struct ResponseWriter {
    res_writer: minihttp::ResponseWriter<TcpStream>,
}

impl ResponseWriter {
    pub fn new(res: minihttp::ResponseWriter<TcpStream>) -> ResponseWriter {
        ResponseWriter { res_writer: res }
    }

    pub fn status(&mut self, status: Status) {
        self.res_writer.status(status)
    }

    pub fn custom_status(&mut self, code: u16, reason: &str) {
        self.res_writer.custom_status(code, reason)
    }

    // TODO(nokaa): currently if the called function returns an error, we are panicking.
    // We want to return the normal value, but the used Error is not exported from minihttp
    pub fn add_header<V: AsRef<[u8]>>(&mut self, name: &str, value: V) {
        if !self.is_started() {
            self.status(Status::Ok);
        }
        self.res_writer.add_header(name, value).unwrap()
    }

    // TODO(nokaa): currently if the called function returns an error, we are panicking.
    // We want to return the normal value, but the used Error is not exported from minihttp
    pub fn format_header<D: Display>(&mut self, name: &str, value: D) {
        self.res_writer.format_header(name, value).unwrap()
    }

    // TODO(nokaa): currently if the called function returns an error, we are panicking.
    // We want to return the normal value, but the used Error is not exported from minihttp
    pub fn add_length(&mut self, n: u64) {
        self.res_writer.add_length(n).unwrap()
    }


    // TODO(nokaa): currently if the called function returns an error, we are panicking.
    // We want to return the normal value, but the used Error is not exported from minihttp
    pub fn add_chunked(&mut self) {
        self.res_writer.add_chunked().unwrap()
    }

    pub fn is_started(&self) -> bool {
        self.res_writer.is_started()
    }

    // TODO(nokaa): currently if the called function returns an error, we are panicking.
    // We want to return the normal value, but the used Error is not exported from minihttp
    pub fn done_headers(&mut self) -> bool {
        self.res_writer.done_headers().unwrap()
    }

    pub fn write_body(&mut self, data: &[u8]) {
        self.res_writer.write_body(data)
    }

    pub fn is_complete(&self) -> bool {
        self.res_writer.is_complete()
    }

    pub fn done<E>(self) -> Finished<IoBuf<TcpStream>, E> {
        self.res_writer.done()
    }

    pub fn steal_socket(self) -> Flushed<TcpStream> {
        self.res_writer.steal_socket()
    }

    pub fn send_file<P: AsRef<Path>>(&mut self, filename: P) -> io::Result<()> {
        let mut file = fs::File::open(filename)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        self.write_all(&buf)
    }

    pub fn redirect(&mut self, status: Status, location: &[u8], data: &[u8]) -> io::Result<()> {
        self.status(status);
        self.add_header("Location", location);
        self.write_all(data)
    }
}

impl Write for ResponseWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = buf.len();
        if !self.is_started() {
            self.status(Status::Ok);
        }
        // TODO(nokaa): We want to make sure that the `Content-Type` header
        // has not already been set.
        self.add_header("Content-Type", b"text/html; charset=utf-8");
        self.add_length(len as u64);
        if self.done_headers() {
            self.write_body(buf);
            Ok((buf.len()))
        } else {
            Ok(0)
        }

    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
