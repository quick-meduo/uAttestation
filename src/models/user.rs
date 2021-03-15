use crate::{config::Constants, models::user_token::UserToken, utils::hash_utils::{hash_password, verify_password}, RangeRequest, KeyRange, EtcdClient};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use etcd_rs::DeleteRequest;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub is_deleted: bool,
    pub created_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
    pub login_session: String,
}

#[derive(Serialize, Deserialize, Clone,Debug)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
    #[serde(default = "default_role")]
    pub role: String,
    #[serde(default = "chrono_now")]
    pub created_at: chrono::NaiveDateTime,
}

fn chrono_now() -> chrono::NaiveDateTime {
    chrono::NaiveDateTime::from_timestamp(Utc::now().timestamp(), 0)
}

fn default_role() -> String {
    DEFAULT_USER_ROLE.to_string()
}

#[derive(Serialize, Deserialize)]
pub struct LoginForm {
    pub username_or_email: String,
    pub password: String,
}

pub struct LoginInfo {
    pub username: String,
    pub role: String,
    pub domain: String,
    pub login_session: String,
}

#[derive(Debug)]
pub struct DeleteUser {
    pub is_deleted: bool,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

pub const DEFAULT_USER_ROLE: &str = "default";

impl User {
    pub async fn find_all() -> Result<Vec<User>, String> {
        let range = RangeRequest::new(KeyRange::prefix("/users"));
        let mut kvs = EtcdClient::new().await.get(range).await;

        let mut user_list: Vec<User> = kvs.take_kvs().iter().map(|kv| -> User{
            let user: User = serde_json::from_str(kv.value_str()).unwrap();
            user
        }).collect();

        Ok(user_list)
    }

    pub async fn find_by_id(id: String) -> Result<User, String> {
        let range = RangeRequest::new(KeyRange::key(format!("/users/{}", id)));
        let mut kvs = EtcdClient::new().await.get(range).await;

        if (kvs.count() > 0) {
            let user: User = serde_json::from_str(kvs.take_kvs().get(0).unwrap().value_str()).unwrap();
            Ok(user)
        } else {
            Err(Constants::MESSAGE_CAN_NOT_FIND_USER.to_string())
        }
    }

    pub async fn delete(id: String) -> Result<String, String> {
        let range = DeleteRequest::new(KeyRange::key(format!("/users/{}", id)));
        let mut result = EtcdClient::new().await.remove(range).await;
        if result.count_deleted() > 0 {
            Ok(Constants::MESSAGE_OK.to_string())
        } else {
            Err(Constants::MESSAGE_DELETE_USER_ERROR.to_string())
        }
    }

    pub async fn is_valid_login_session(user_token: &UserToken) -> bool {
        let result = User::find_by_id(user_token.user.clone()).await;

        match result {
            Ok(user) => true,
            Err(_) => false
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_rt;
    use crate::models::user::User;

    #[actix_rt::test]
    pub async fn test_find_all(){
        let all =  User::find_all().await.unwrap();
        for u in all {
            println!("{:?}",u);
        }
    }

    #[actix_rt::test]
    pub async fn test_find_id(){
        let user =  User::find_by_id("10000".to_string()).await.unwrap();
        println!("{:?}",user);
    }

    #[actix_rt::test]
    pub async fn delete_user_id(){
        let result =  User::delete("10000".to_string()).await.unwrap();
        println!("{:?}",result);
    }
}
