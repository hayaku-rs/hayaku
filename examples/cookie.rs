extern crate hayaku;

use hayaku::{Http, Router, Request, ResponseWriter, Status, Cookie};
use std::rc::Rc;

#[derive(Clone)]
struct Context {
    pub username: String,
    pub password: String,
}

fn main() {
    let ctx = Context {
        username: "username".to_string(),
        password: "password".to_string(),
    };

    let mut router = Router::new();
    router.get("/", Rc::new(home_handler)).unwrap();
    router.post("/", Rc::new(login_handler)).unwrap();
    router.get("/secret", Rc::new(secret_handler)).unwrap();

    let addr = "127.0.0.1:3000".parse().unwrap();
    let http = Http::new(router, ctx);
    http.listen_and_serve(addr);
}

fn home_handler(_req: &Request, res: &mut ResponseWriter, _ctx: &Context) {
    res.send_file("examples/login.html");
}

fn login_handler(req: &Request, res: &mut ResponseWriter, ctx: &Context) {
    let username = req.form_value("username").unwrap();
    let password = req.form_value("password").unwrap();

    if username == ctx.username && password == ctx.password {
        let cookie = Cookie::new("logged in", "true");
        res.set_cookie(cookie);
        res.redirect(Status::Found, b"/secret", b"You logged in!");
    } else {
        res.redirect(Status::Found, b"/", b"Incorrect login");
    }
}

fn secret_handler(req: &Request, res: &mut ResponseWriter, _ctx: &Context) {
    let cookies = req.get_cookies();
    println!("cookies: {:?}", cookies);
    res.send_file("examples/secret.html");
}
