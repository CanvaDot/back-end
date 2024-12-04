use actix_web::{post, web::Form, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct LoginParams {
    email: String,
    password: String
}

#[post("/auth/login")]
pub async fn login(_params: Form<LoginParams>) -> impl Responder {
    HttpResponse::Ok()
}
