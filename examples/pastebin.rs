#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rand;
extern crate hayaku;

use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use hayaku::{Http, Request, ResponseWriter, Router, Status};
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

    let mut router = Router::new();
    router.get("/", Rc::new(new_paste)).unwrap();
    router.post("/new", Rc::new(make_paste)).unwrap();
    router.get("/:pastename", Rc::new(get_paste)).unwrap();
    let http = Http::new(router, ctx);
    // http.handle_func(Path::from(String::from("/404")), Rc::new(not_found));
    http.listen_and_serve(addr);
}

fn new_paste(_req: &Request, res: &mut ResponseWriter, _ctx: &Ctx) {
    if let Err(e) = res.send_file("examples/new.html") {
        error!("{}", e);
    }
}

fn get_paste(req: &Request, res: &mut ResponseWriter, ctx: &Ctx) {
    let path = req.path;
    info!("path: {}", path);

    // Obtain a read lock on the context and read from the database
    // sending the results if found, otherwise sending a 404
    let lock = ctx.read().unwrap();
    if let Some(p) = lock.db.get(&path[1..]) {
        res.add_header("Content-Type", b"text/plain; charset=utf-8");
        res.write_all(p.as_bytes()).unwrap();
    } else {
        not_found(req, res, ctx);
    }
}

fn make_paste(req: &Request, res: &mut ResponseWriter, ctx: &Ctx) {
    // Retrive the submitted form data
    let filetype = req.form_value("filetype".to_string()).unwrap();
    let paste = req.form_value("paste".to_string()).unwrap();

    // Create the name of this paste to store in our database.
    // Name takes the form of [a-zA-Z0-9]+.{filetype}
    let mut name = gen_paste_name();
    name.push('.');
    name.push_str(&filetype);


    // Obtain a write lock on the context and insert the paste
    // into our db
    let mut lock = ctx.write().unwrap();
    lock.db.insert(String::from(name.clone()), paste.clone());

    // Redirect the user to the url of the created paste
    if let Err(e) = res.redirect(Status::Found, name.as_bytes(), b"You are being redirected") {
        error!("{}", e);
    }
}

fn not_found(_req: &Request, res: &mut ResponseWriter, _ctx: &Ctx) {
    res.status(Status::NotFound);
    info!("{}", res.is_started());
    if let Err(e) = res.write_all(b"404 - Page not found") {
        error!("{}", e);
    }
}

/// Generate a unique id of length 10 from the set of ascii characters
fn gen_paste_name() -> String {
    rand::thread_rng().gen_ascii_chars().take(10).collect()
}
