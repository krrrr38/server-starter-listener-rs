use actix_web::{web, App, HttpResponse, HttpServer};

use server_starter_listener::{listeners, ServerStarterListener};

/// ## Example
///
/// ```sh
/// > cargo build --examples
/// > start_server --port=8000 --pid-file=/tmp/actix-web-example.pid -- ./target/debug/actix-web-example
/// > curl -i localhost:8000/hello
/// > kill -SIGHUP `cat /tmp/actix-web-example.pid` # hot deploy
/// ```
fn main() {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let builder = HttpServer::new(|| {
        App::new().service(web::resource("/hello").route(web::get().to(|| {
            log::info!("pid {:?}", std::process::id());
            return HttpResponse::Ok();
        })))
    });

    let builder = listeners()
        .unwrap()
        .into_iter()
        .fold(builder, |builder, listener| {
            match listener {
                ServerStarterListener::Tcp(listener) => builder.listen(listener).unwrap(),
                ServerStarterListener::Uds(listener) => {
                    // listen_uds required actix-web "uds" features
                    builder.listen_uds(listener).unwrap()
                }
            }
        });

    builder.run().unwrap();
}
