use std::io::Result as IoResult;
use actix_web::{App, HttpServer, Scope};
use routes::{auth::{login::login, register::register, user::user, activate::activate}, socket::session};
use tokio::main;

mod helpers;
mod models;
mod routes;

#[main]
async fn main() -> IoResult<()> {
    HttpServer::new(|| {
        App::new()
            .service(session)
            .service(
                Scope::new("/auth")
                    .service(login)
                    .service(register)
                    .service(user)
                    .service(activate)
            )
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
