use std::io::Result as IoResult;
use actix_web::{App, HttpServer};
use routes::socket::session;
use tokio::main;

mod helpers;
mod models;
mod routes;

#[main]
async fn main() -> IoResult<()> {
    HttpServer::new(|| {
        App::new()
            .service(session)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
