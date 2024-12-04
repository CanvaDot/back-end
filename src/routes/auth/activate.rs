use actix_web::{post, web::Query, HttpResponse, Responder};
use serde::Deserialize;
use crate::{grv, models::user::User};


#[derive(Deserialize)]
struct ActivateParams {
    t: String
}

#[post("/activate")]
pub async fn activate(params: Query<ActivateParams>) -> impl Responder {
    grv!(User::activate(&params.t).await);

    HttpResponse::Ok()
        .finish()
}
