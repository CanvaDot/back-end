use std::ops::Add;
use actix_web::{cookie::{time::{Duration, OffsetDateTime}, Cookie}, post, web::Form, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::{grv, models::user::User};

#[derive(Serialize, Deserialize)]
struct LoginParams {
    email: String,
    password: String
}

#[post("/login")]
pub async fn login(params: Form<LoginParams>) -> impl Responder {
    let LoginParams { email, password } = params.into_inner();

    let user = match grv!(User::login(email, password).await) {
        Some(user) => user,
        None => {
            return HttpResponse::Unauthorized()
                .body("The account does not exist.");
        }
    };

    let mut cookie = Cookie::new("Session", grv!(user.jwt()));

    cookie.set_expires(
        OffsetDateTime::now_utc()
            .add(Duration::hours(12))
    );

    cookie.set_path("/");

    HttpResponse::Ok()
        .cookie(cookie)
        .finish()
}
