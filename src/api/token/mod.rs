use actix_web::{web,HttpResponse, Responder, post, get};
use serde::{Serialize,Deserialize};
use crate::config::Constants;

use serde_json::Value;
use serde_json::json;
use chrono::Utc;
use crate::models::response::ResponseBody;
use crate::models::user::LoginInfo;
use crate::models::user_token::UserToken;

#[derive(Deserialize)]
pub struct token_request {
    pub name: String,
    pub email: String,
    pub domain: String,
}

#[post("/token/create")]
pub async fn create(body: web::Json<token_request>) -> impl Responder {
    //TODO: support role check
    let login = LoginInfo {
        username: body.name.clone(),
        role: "default".to_string(),
        domain: body.domain.clone(),
        login_session: "10010".to_string()
    };

    let token = UserToken::generate_token(login);
    let data: Value = json!({
      "token": token
    });

    //TODO: persist it in ETCD

    HttpResponse::Ok().json(
        ResponseBody::new(Constants::MESSAGE_OK,data)
    )
}