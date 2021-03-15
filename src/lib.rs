#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#[macro_use]
#[warn(unused_imports)]
pub extern crate rbatis;
pub use rbatis::{rbatis::Rbatis,plugin::page,plugin::page::PageRequest,plugin::page::Page};

#[macro_use]
pub extern crate lazy_static;

pub use etcd_rs::*;
pub use actix_web::{get, post, guard,middleware::Logger,http::header, HttpResponse, HttpServer};
pub use actix_web::Responder;
pub use actix_web::HttpRequest;
pub use actix_web::App;
pub use actix_web::web;
pub use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
pub use std::env;
pub use std::sync::Mutex;
pub use actix_cors::Cors;


pub use rbatis::crud::CRUD;
pub use futures::TryFutureExt;
pub use log::kv::Source;

pub use crate::config::{Configurations,Constants};
pub use crate::config::Configurations::ConfigResult;
pub use std::{borrow::BorrowMut, collections::HashMap};
pub use crate::db::ectd::EtcdClient;
pub use log::{debug, error, info, trace, warn};

//add links
pub mod auth;
pub mod config;
pub mod api;
pub mod models;
pub mod db;
pub mod utils;
pub mod errors;
pub mod routers;

use std::future::Future;

lazy_static! {
  // Rbatis是线程、协程安全的，运行时的方法是Send+Sync，无需担心线程竞争
  // pub static ref RB: Rbatis = Rbatis::new();
  pub static ref CONFIG: Configurations::AttestationConfig = Configurations::AttestationConfig::new();
}
