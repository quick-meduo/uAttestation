use etcd_rs;
use etcd_rs::{ClientConfig, PutRequest, RangeRequest, DeleteRequest};
use crate::{PutResponse, RangeResponse, DeleteResponse};

pub struct EtcdClient(etcd_rs::Client);

impl EtcdClient {
    pub async fn new() -> EtcdClient {
        let endponts = crate::CONFIG.get_array("etcd.bind").unwrap();

        let servers: Vec<String> = endponts.into_iter().map(|s| {
            s.to_string()
        }).collect();

        //TODO: need add TLS support and Authication
        let client = etcd_rs::Client::connect(ClientConfig {
            auth: None,
            endpoints: servers,
            tls: None,
        });

        EtcdClient {
            0: client.await.unwrap()
        }
    }

    pub async fn put(&self, req: PutRequest) -> PutResponse{
        let response = self.0.kv().put(req).await;
        response.unwrap()
    }

    pub async fn get(&self, range: RangeRequest) -> RangeResponse {
        let response = self.0.kv().range(range).await;
        response.unwrap()
    }

    pub async fn remove(&self, req: DeleteRequest) -> DeleteResponse {
        let response = self.0.kv().delete(req).await;
        response.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;
    use actix_rt;
    use etcd_rs::{RangeRequest, KeyRange, DeleteRequest, Client, PutRequest};
    use crate::models::user::User;
    use chrono::Utc;

    #[test]
    fn async_block() {
        assert_eq!(4, block_on(async { 4 }));
    }

    async fn lens() -> i32 {
        4
    }

    #[actix_rt::test]
    async fn test_str_len_async() {
        assert_eq!(lens().await, 4);
    }

    #[test]
    fn test_connect() {
        block_on(async {
            let cli = EtcdClient::new().await;
            // cli.kv().put(PutRequest{
            //     proto:
            // })
        })
    }

    #[actix_rt::test]
    async fn test_async() {
        let cli = EtcdClient::new().await;

        let user = User {
            id: 100001,
            username: "anonymous".to_string(),
            email: "anonymous@quick.org".to_string(),
            password: "@meduo".to_string(),
            role: "0".to_string(),
            is_deleted: false,
            created_at: chrono::NaiveDateTime::from_timestamp(Utc::now().timestamp(), 0),
            deleted_at: None,
            login_session: "SESSION-1000100".to_string()
        };

        let s = serde_json::to_string(&user).unwrap();


        let req = PutRequest::new("/users/anonymous",s);
        cli.put(req).await;

        let range =  RangeRequest::new(KeyRange::key("/users/20001"));
        let mut result = cli.get(range).await;

        let kvs = result.take_kvs();

        for i in kvs{
            let user: User = serde_json::from_str(i.value_str()).unwrap();
            println!("{:?}",user);
        }
    }
}
