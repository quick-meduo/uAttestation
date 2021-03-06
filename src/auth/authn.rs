use actix_casbin_auth::CasbinVals;
use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    http::{HeaderName, HeaderValue, Method},
    web::Data,
    Error, HttpMessage, HttpResponse,
};
use futures::{
    future::{ok, Ready},
    Future,
};

use crate::config::Constants;

use std::cell::RefCell;
use std::rc::Rc;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::models::response::ResponseBody;
use crate::utils::token_utils;

use crate::EtcdClient;
pub use log::{debug, error, info, trace, warn};

pub struct Authentication;

impl<S, B> Transform<S> for Authentication
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthenticationMiddleware {
            service: Rc::new(RefCell::new(service)),
        })
    }
}
pub struct AuthenticationMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service for AuthenticationMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let mut srv = self.service.clone();
        let mut authenticate_pass: bool = false;
        let mut public_route: bool = false;
        let mut authenticate_username: String = String::from("");
        let mut domain: String = String::from("");

        if Method::OPTIONS == *req.method() {
            info!("HTTP OPTIONS");
            authenticate_pass = true;
            authenticate_username = "option".to_string();
            domain = "public".to_string();
        } else {
            for ignore_route in Constants::IGNORE_ROUTES.iter() {
                if req.path().starts_with(ignore_route) {
                    authenticate_pass = true;
                    public_route = true;
                }
            }
            if !authenticate_pass {
                if let Some(authen_header) = req.headers().get(Constants::AUTHORIZATION) {
                    if let Ok(authen_str) = authen_header.to_str() {
                        if authen_str.starts_with("bearer") || authen_str.starts_with("Bearer") {
                            let token = authen_str[7..authen_str.len()].trim();
                            if let Ok(token_data) = token_utils::decode_token(token.to_string()) {
                                if token_utils::verify_token(&token_data).is_ok() {
                                    authenticate_username = token_data.claims.user;
                                    domain = token_data.claims.domain;
                                    authenticate_pass = true;
                                } else {
                                    error!("Invalid token");
                                }
                            }
                        }
                    }
                }
            }
        }

        if authenticate_pass {
            if public_route {
                let vals = CasbinVals {
                    subject: "anonymous".to_string(),
                    domain: Some("public".to_string()),
                };
                req.extensions_mut().insert(vals);
                Box::pin(async move { srv.call(req).await })
            } else {
                let vals = CasbinVals {
                    subject: authenticate_username,
                    domain: Some(domain),
                };
                req.extensions_mut().insert(vals);
                Box::pin(async move { srv.clone().call(req).await })
            }
        } else {
            Box::pin(async move {
                Ok(req.into_response(
                    HttpResponse::Unauthorized()
                        .json(ResponseBody::new(
                            Constants::MESSAGE_INVALID_TOKEN,
                            Constants::EMPTY,
                        ))
                        .into_body(),
                ))
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use casbin::prelude::*;
    use actix_rt;
    use std::sync::{Arc, RwLock};
    use casbin::DefaultRoleManager;
    use actix_casbin::casbin::{DefaultModel, FileAdapter, Result, CachedEnforcer};
    use actix_casbin::{CasbinActor, CasbinCmd, CasbinResult};
    use actix_casbin_auth::CasbinService;
    use actix::{Actor, Supervisor};
    use actix_casbin_auth::casbin::function_map::key_match2;


    #[actix_rt::test]
    pub async fn test_policy(){
        let mut e = Enforcer::new(
            "examples/rbac_with_domains_model.conf",
            "examples/rbac_with_domains_policy.csv",
        )
            .await
            .unwrap();

        let new_rm = Arc::new(RwLock::new(DefaultRoleManager::new(10)));

        e.set_role_manager(new_rm).unwrap();

        assert!(e.enforce(("alice", "domain1", "data1", "read")).unwrap(),);
        assert!(e.enforce(("alice", "domain1", "data1", "write")).unwrap());
        assert!(e.enforce(("bob", "domain2", "data2", "read")).unwrap());
        assert!(e.enforce(("bob", "domain2", "data2", "write")).unwrap());
    }

    #[actix_rt::test]
    pub async fn test_policy_conf(){
        let mut e = Enforcer::new(
            "policy/casbin.conf",
            "policy/preset_policy.csv",
        )
            .await
            .unwrap();

        let new_rm = Arc::new(RwLock::new(DefaultRoleManager::new(10)));

        e.set_role_manager(new_rm).unwrap();
        assert!(e.enforce(("anonymous", "public", "/api/echo", "POST")).unwrap());
        assert!(e.enforce(("admin", "publicxx", "/api/xx", "x")).unwrap());
    }

    #[actix_rt::test]
    async fn test_policy_auth_conf() -> Result<()> {
        let model = DefaultModel::from_file("policy/casbin.conf").await.unwrap();
        let adapter = casbin::FileAdapter::new("policy/preset_policy.csv");

        let mut casbin_middleware = CasbinService::new(model, adapter).await.unwrap();
        casbin_middleware
            .write()
            .await
            .get_role_manager()
            .write()
            .unwrap()
            .matching_fn(Some(key_match2), None);

        let share_enforcer = casbin_middleware.get_enforcer();
        let clone_enforcer = share_enforcer.clone();
        let casbin_actor = CasbinActor::<CachedEnforcer>::set_enforcer(share_enforcer).unwrap();
        // let started_actor = Supervisor::start(|_| casbin_actor);

        assert!(clone_enforcer.read().await.enforce(("anonymous", "public", "/api/echo", "POST")).unwrap());
        assert!(clone_enforcer.read().await.enforce(("admin", "publicxx", "/api/xx", "x")).unwrap());
        let b = matches!(clone_enforcer.read().await.enforce(("aaadmin", "publicxx", "/api/xx", "x")).unwrap(),false);
        println!("This is {}",b);
        Ok(())
    }
}