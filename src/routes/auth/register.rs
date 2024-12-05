use actix_web::{cookie::{time::{Duration, OffsetDateTime}, Cookie}, post, web::Form, HttpResponse, Responder};
use email_address::EmailAddress;
use serde::Deserialize;
use crate::{grv, models::user::User};
use std::ops::Add;

#[derive(Deserialize)]
struct RegisterParams {
    username: String,
    email: String,
    password: String
}

#[post("/register")]
pub async fn register(params: Form<RegisterParams>) -> impl Responder {
    let RegisterParams { username, email, password } = params.into_inner();

    if username.len() > 25 || email.len() > 75 {
        return HttpResponse::BadRequest()
            .body("Max 25 characters per username and 75 characters per email allowed.");
    }

    if !EmailAddress::is_valid(&email) {
        return HttpResponse::BadRequest()
            .body("The email is not a valid email.");
    }

    if username.chars().any(|c| !c.is_alphanumeric() || c.is_whitespace()) {
        return HttpResponse::BadRequest()
            .body("Username must only contain readable characters");
    }

    if grv!(User::exists(&username, &email).await) {
        return HttpResponse::BadRequest()
            .body("This username or email already exists");
    }

    let user = grv!(User::insert(email, username, password).await);
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
