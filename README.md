# hayaku

An http library based on Golang's `net/http` written using Tokio and Futures.

### NOTE: This is pre-alpha software relying on pre-alpha software. Use at your own risk.

### Using
Note that hayaku relies on serde, which uses unstable compiler features. This
means that until these features are stabilized, you must use a nightly version
of the rust compiler.

Place `hayaku = { git="https://github.com/nokaa/hayaku" }` in your `Cargo.toml`.

```Rust
extern crate hayaku;
use haykau::{Http, Router, Request, ResponseWriter};
use std::rc::Rc;

fn main () {
    let mut router = Router::new();
    router.get("/", Rc::new(home_handler)).unwrap();

    let http = Http::new(router, ());
    http.listen_and_serve("127.0.0.1:3000".parse().unwrap());
}

fn home_handler(_req: Request, res; ResponseWriter, _ctx: &()) {
    res.write_all(b"Hello, world!").unwrap();
}
```
