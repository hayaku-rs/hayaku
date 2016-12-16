#[macro_use]
extern crate log;
extern crate env_logger;
extern crate hayaku;

use hayaku::{Http, Router, Request, Response, Status, Cookie};
use std::sync::Arc;

#[derive(Clone)]
struct Context {
    pub username: String,
    pub password: String,
}

fn main() {
    env_logger::init();
    let ctx = Context {
        username: "username".to_string(),
        password: "password".to_string(),
    };

    let mut router = Router::new();
    router.get("/", Arc::new(home_handler)).unwrap();
    router.post("/login", Arc::new(login_handler)).unwrap();
    router.get("/secret", Arc::new(secret_handler)).unwrap();

    let addr = "127.0.0.1:3000".parse().unwrap();
    let http = Http::new(router, ctx);
    http.listen_and_serve(addr);
}

fn home_handler(_req: &Request, res: &mut Response, _ctx: &Context) {
    res.send_file("examples/assets/login.html");
}

fn login_handler(req: &Request, res: &mut Response, ctx: &Context) {
    let username = req.form_value("username").unwrap();
    let password = req.form_value("password").unwrap();

    if username == ctx.username && password == ctx.password {
        let p = "/".to_string();
        let cookie = Cookie::new("loggedin", "true").path(p);
        res.set_cookie(&cookie);
        res.redirect(Status::Found, "/secret", b"");
    } else {
        res.redirect(Status::Found, "/", b"Incorrect login");
    }
}

fn secret_handler(req: &Request, res: &mut Response, _ctx: &Context) {
    let cookies = req.get_cookies();
    info!("cookies: {:?}", cookies);
    for cookie in cookies {
        if cookie.name == "loggedin" && cookie.value == "true" {
            res.send_file("examples/assets/secret.html");
            return;
        }
    }

    res.redirect(Status::Found, "/", b"Incorrect login");
}
