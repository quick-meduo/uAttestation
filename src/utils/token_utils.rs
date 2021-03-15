use crate::{models::{
    user::User,
    user_token::{UserToken, KEY},
}, EtcdClient};
use actix_web::web::Data;
use jsonwebtoken::{DecodingKey, TokenData, Validation};
use futures::executor::block_on;

pub fn decode_token(
    token: String,
) -> jsonwebtoken::errors::Result<TokenData<UserToken>> {
    jsonwebtoken::decode::<UserToken>(
        &token,
        &DecodingKey::from_secret(&KEY),
        &Validation::default(),
    )
}

pub fn verify_token(
    token_data: &TokenData<UserToken>,
) -> Result<String, String> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();

    match rt.block_on(
        User::is_valid_login_session(&token_data.claims)
    ) {
        true => Ok(token_data.claims.user.to_string()),
        false => Err("Invalid token".to_string())
    }
}
