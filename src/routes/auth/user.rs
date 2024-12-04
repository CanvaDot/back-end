use actix_web::{get, HttpResponse, Responder};

use crate::models::user::User;

#[get("/user")]
pub async fn user(user: User) -> impl Responder {
    HttpResponse::Ok()
        .json(user)
}
