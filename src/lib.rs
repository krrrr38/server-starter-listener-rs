//! Get Server::Starter listeners for rust application
//!
//! This crate providers [start_server](https://github.com/lestrrat-go/server-starter) / [start_server](https://metacpan.org/pod/start_server) listeners for rust server applications.
//!
//! # Examples
//!
//! ```no_run
//! use actix_web::{HttpServer, App};
//! use server_starter_listener::{listeners, ServerStarterListener};
//!
//! let listener = listeners().unwrap().pop().unwrap();
//! match listener {
//!   ServerStarterListener::Tcp(listener) => {
//!     HttpServer::new(|| App::new()).listen(listener).unwrap().run().unwrap();
//!   }
//!   _ => unimplemented!(),
//! }
//! ```
//!
//! You need to start application using [start_server](https://github.com/lestrrat-go/server-starter) / [start_server](https://metacpan.org/pod/start_server).
//!
//! ```sh
//! > start_server --port=80 -- your_server_binary
//! ```
//!
//! Now you can do hot-deploy by send `SIGHUP` to `start_server` process.
//! `start_server` share file descriptor to new process and send `SIGTERM` to old process.
//!

#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;

use std::net::TcpListener;
use std::os::unix::io::{FromRawFd, RawFd};
use std::os::unix::net::UnixListener;

use regex::Regex;

const SERVER_STARTER_PORT_ENV: &str = "SERVER_STARTER_PORT";

lazy_static! {
    static ref HOST_PORT_REGEX: Regex = Regex::new("^[^:]+:\\d+$").unwrap();
    static ref PORT_REGEX: Regex = Regex::new("^\\d+$").unwrap();
}

///
/// Kind of server starter listener
///
#[derive(Debug)]
pub enum ServerStarterListener {
    Tcp(TcpListener),
    Uds(UnixListener),
}

impl ServerStarterListener {
    fn tcp(fd: RawFd) -> ServerStarterListener {
        ServerStarterListener::Tcp(unsafe { TcpListener::from_raw_fd(fd) })
    }

    fn uds(fd: RawFd) -> std::io::Result<ServerStarterListener> {
        Ok(ServerStarterListener::Uds(unsafe {
            UnixListener::from_raw_fd(fd)
        }))
    }
}

///
/// A server starter listener error
///
#[derive(Fail, Debug)]
pub enum ListenerError {
    #[fail(display = "server starter port env var not found.")]
    ServerStarterPortEnvNotFound,
    #[fail(display = "cannot parse server starter port: {}", _0)]
    InvalidServerStarterPortSpec(String),
    #[fail(display = "failed to bind uds: {}", _0)]
    UnixListenerBindError(#[fail(cause)] std::io::Error),
}

///
/// Get server starter listening listeners.
///
/// There are tcp and unix domain socket listeners.
///
/// # Errors
///
/// Returns as `ListenerError` if `SERVER_STARTER_PORT` env var is not found or invalid format.
///
pub fn listeners() -> Result<Vec<ServerStarterListener>, ListenerError> {
    let specs = match std::env::var(SERVER_STARTER_PORT_ENV) {
        Ok(specs) => specs,
        Err(_) => return Err(ListenerError::ServerStarterPortEnvNotFound),
    };

    let specs: Vec<&str> = specs.split(";").collect();
    let mut results = vec![];
    for spec in specs {
        let pair: Vec<&str> = spec.split("=").collect();
        if pair.len() != 2 {
            return Err(ListenerError::InvalidServerStarterPortSpec(spec.into()));
        }

        let (left, fd) = (pair[0], pair[1]);
        let fd: i32 = match fd.parse() {
            Ok(fd) => fd,
            Err(_) => return Err(ListenerError::InvalidServerStarterPortSpec(spec.into())),
        };

        if let Some(_) = HOST_PORT_REGEX.find(left) {
            results.push(ServerStarterListener::tcp(fd));
        } else if let Some(_) = PORT_REGEX.find(left) {
            results.push(ServerStarterListener::tcp(fd));
        } else {
            let uds_listener = match ServerStarterListener::uds(fd) {
                Ok(uds_listener) => uds_listener,
                Err(e) => return Err(ListenerError::UnixListenerBindError(e)),
            };
            results.push(uds_listener);
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use std::os::unix::io::AsRawFd;

    use crate::{listeners, ServerStarterListener};

    #[test]
    fn listeners_tcp() {
        let assert_tcp_listener = |var, fd| {
            std::env::set_var("SERVER_STARTER_PORT", var);
            let results = listeners();
            match results {
                Ok(results) => {
                    assert_eq!(1, results.len());
                    let listener = results.first().unwrap();
                    match listener {
                        ServerStarterListener::Tcp(tcp_listener) => {
                            assert_eq!(fd, tcp_listener.as_raw_fd());
                        }
                        ServerStarterListener::Uds(_) => {
                            assert!(false, "not tcp listener {:?}", listener)
                        }
                    }
                }
                Err(_) => assert!(false, "results not ok {:?}", results),
            }
        };

        assert_tcp_listener("80=2", 2);
        assert_tcp_listener("127.0.0.1:8080=3", 3);
        assert_tcp_listener("localhost:8080=4", 4);
    }

    #[test]
    fn listeners_uds() {
        let assert_uds_listener = |var, fd| {
            std::env::set_var("SERVER_STARTER_PORT", var);
            let results = listeners();
            match results {
                Ok(results) => {
                    assert_eq!(1, results.len());
                    let listener = results.first().unwrap();
                    match listener {
                        ServerStarterListener::Tcp(_) => {
                            assert!(false, "not uds listener {:?}", listener)
                        }
                        ServerStarterListener::Uds(uds_listener) => {
                            assert_eq!(fd, uds_listener.as_raw_fd());
                        }
                    }
                }
                Err(_) => assert!(false, "results not ok {:?}", results),
            }
        };

        assert_uds_listener("/tmp/server-starter-listener/server.sock=2", 2);
    }

    #[test]
    fn listeners_without_env() {
        std::env::remove_var("SERVER_STARTER_PORT");
        assert!(listeners().is_err());
    }

    #[test]
    fn listeners_invalid_env() {
        std::env::set_var("SERVER_STARTER_PORT", "80=a");
        assert!(listeners().is_err());
    }
}
