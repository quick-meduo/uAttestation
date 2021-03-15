use crate::models::user::LoginInfo;
use crate::utils::token_utils;
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, TokenData};
use serde::{Deserialize, Serialize};

pub static KEY: [u8; 16] = *std::include_bytes!("../config/secret.key");
static ONE_WEEK: i64 = 60 * 60 * 24 * 7; // in seconds

#[derive(Serialize, Deserialize)]
pub struct UserToken {
    // issued at
    pub iat: i64,
    // expiration
    pub exp: i64,
    // data
    pub user: String,
    pub role: String,
    pub domain: String,
    pub login_session: String,
}

impl UserToken {
    pub fn generate_token(login: LoginInfo) -> String {
        let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nanosecond -> second
        let payload = UserToken {
            iat: now,
            exp: now + ONE_WEEK,
            user: login.username,
            role: login.role,
            domain: login.domain,
            login_session: login.login_session,
        };

        jsonwebtoken::encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(&KEY),
        )
        .unwrap()
    }

    pub fn decode(token: String) -> UserToken {
        token_utils::decode_token(token).unwrap().claims
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test() {
        let login = LoginInfo {
            username: "anonymous".to_string(),
            role: "".to_string(),
            domain: "public".to_string(),
            login_session: "10010".to_string()
        };

        let token = UserToken::generate_token(login);
        println!("{}",token);

        let login =  UserToken::decode(token.clone());
    }
}