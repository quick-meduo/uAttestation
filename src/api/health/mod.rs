use actix_web::{HttpResponse, Responder, post, get};
use chrono::Utc;

#[post("/echo")]
pub async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/alive")]
pub async fn alive() -> impl Responder {
    let timestamp :String = chrono::NaiveDateTime::from_timestamp(Utc::now().timestamp(), 0).to_string();
    format!("Server Time at {}",timestamp)
}

