[![Crates.io](https://img.shields.io/crates/v/cargo-readme.svg)](https://crates.io/crates/server-starter-listener)
[![Build Status](https://travis-ci.org/krrrr38/server-starter-listener-rs.svg?branch=master)](https://travis-ci.org/krrrr38/server-starter-listener-rs)

# server-starter-listener-rs

Get Server::Starter listeners for rust application

This crate providers [start_server](https://github.com/lestrrat-go/server-starter) / [start_server](https://metacpan.org/pod/start_server) listeners for rust server applications.

## Examples

```rust
use actix_web::{HttpServer, App};
use server_starter_listener::{listeners, ServerStarterListener};

let listener = listeners().unwrap().pop().unwrap();
match listener {
  ServerStarterListener::Tcp(listener) => {
    HttpServer::new(|| App::new()).listen(listener).unwrap().run().unwrap();
  }
  _ => unimplemented!(),
}
```

You need to start application using [start_server](https://github.com/lestrrat-go/server-starter) / [start_server](https://metacpan.org/pod/start_server).

```sh
> start_server --port=80 -- your_server_binary
```

Now you can do hot-deploy by send `SIGHUP` to `start_server` process.
`start_server` share file descriptor to new process and send `SIGTERM` to old process.


Current version: 0.1.0

Some additional info here

License: MIT OR Apache-2.0
