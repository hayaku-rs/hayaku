#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rand;
extern crate regex;
extern crate hayaku;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use hayaku::{Http, Request, ResponseWriter, forms, util};
use regex::Regex;
use rand::Rng;

type Ctx = Arc<RwLock<Context>>;

#[derive(Clone)]
struct Context {
    db: HashMap<String, String>,
}

impl Context {
    fn new() -> Context {
        Context { db: HashMap::new() }
    }
}

fn main() {
    env_logger::init().unwrap();
    info!("Starting up");
    let addr = "127.0.0.1:3000".parse().unwrap();
    let context = Context::new();
    let ctx = Arc::new(RwLock::new(context));

    let mut http = Http::new(ctx);
    http.handle_func(Regex::new(r"/").unwrap(), Rc::new(new_paste));
    http.handle_func(Regex::new(r"/new").unwrap(), Rc::new(make_paste));
    http.handle_func(Regex::new(r"/[a-zA-Z0-9]+").unwrap(), Rc::new(get_paste));
    http.handle_func(Regex::new(r"/404").unwrap(), Rc::new(not_found));
    http.listen_and_serve(addr);
}

fn new_paste(_req: &Request, res: &mut ResponseWriter, _ctx: &Ctx) {
    if let Err(e) = util::send_file(res, "examples/new.html") {
        error!("{}", e);
    }
}

fn get_paste(req: &Request, res: &mut ResponseWriter, ctx: &Ctx) {
    let ref path = req.path;
    info!("path: {}", path);

    // Obtain a read lock on the context and read from the database
    // sending the results if found, otherwise sending a 404
    let lock = ctx.read().unwrap();
    if let Some(p) = lock.db.get(&path[1..]) {
        util::send_string_raw(res, p.as_bytes());
    } else {
        not_found(req, res, ctx);
    }
}

fn make_paste(req: &Request, res: &mut ResponseWriter, ctx: &Ctx) {
    // Get the contents of the request body
    let buf = match req.body {
        Some(ref b) => {
            &b.data[..]
        }
        None => panic!("no body found"),
    };

    // Parse the body as urlencoded form data
    let form = forms::parse_form(&buf).unwrap();

    let filetype = form.get(&"filetype".to_string()).unwrap();
    let paste = form.get(&"paste".to_string()).unwrap();

    // Create the name of this paste to store in our database.
    // Name takes the form of [a-zA-Z0-9]+.{filetype}
    let mut name = gen_paste_name();
    let filetype = ::std::str::from_utf8(filetype).unwrap();
    name.push('.');
    name.push_str(filetype);

    let paste_str = ::std::str::from_utf8(paste).unwrap();

    // Obtain a write lock on the context and insert the paste
    // into our db
    let mut lock = ctx.write().unwrap();
    lock.db.insert(String::from(name.clone()), String::from(paste_str));

    // Redirect the user to the url of the created paste
    if let Err(e) = util::redirect(res, b"You are being redirected", name.as_bytes(), 302) {
        error!("{}", e);
    }
}

fn not_found(_req: &Request, res: &mut ResponseWriter, _ctx: &Ctx) {
    if let Err(e) = util::error(res, b"404 - Page not found", 404) {
        error!("{}", e);
    }
}

/// Generate a unique id of length 10 from the set of ascii characters
fn gen_paste_name() -> String {
    let s: String = rand::thread_rng().gen_ascii_chars().take(10).collect();
    s
}