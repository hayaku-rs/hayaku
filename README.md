# hayaku

An http library based on Golang's `net/http` written using Tokio and Futures.

### Warning: This is pre-alpha software relying on pre-alpha software. Use at your own risk.

Various example servers are included in `examples/`. For a more complex use case,
take a look at [neppit]("https://github.com/nokaa/neppit").

### Using
Note that hayaku relies on serde, which uses unstable compiler features. This
means that until these features are stabilized, you must use a nightly version
of the rust compiler.

Place `hayaku = { git="https://github.com/nokaa/hayaku" }` in your `Cargo.toml`.

```Rust
extern crate hayaku;
use haykau::{Http, Router, Request, Response};
use std::sync::Arc;

fn main () {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let mut router = Router::new();
    router.get("/", Arc::new(home_handler)).unwrap();

    Http::new(router, ()).listen_and_serve(addr);
}

fn home_handler(_req: &Request, res: &mut Response, _ctx: &()) {
    res.body(b"Hello, world!").unwrap();
}
```
