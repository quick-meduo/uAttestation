#[warn(unused_imports)]
use uAttestation::*;

#[macro_use]
pub extern crate lazy_static;

#[macro_use]
pub extern crate rbatis;

use actix_web::middleware::normalize::TrailingSlash;
use actix_web::middleware::Logger;
use actix_web::middleware::NormalizePath;
use actix_casbin_auth::CasbinService;
use actix_casbin::casbin::{
    function_map::key_match2, CachedEnforcer, CoreApi, DefaultModel, MgmtApi, Result,
};
use uAttestation::utils::csv_utils::{load_csv, walk_csv};
use actix_casbin::CasbinActor;
use casbin::FileAdapter;
use actix::Supervisor;
use actix_cors::Cors;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match CONFIG.get_bool("application.debug.enable") {
        Ok(enable) if enable  => {
            if let Ok(_result) = fast_log::init_log("requests.log", 1000,log::Level::Info,None,true){
                println!("Initilaized log at INFO level successfully");
            }
        },
        _ => {
            if let Ok(_result) = fast_log::init_log("requests.log", 1000,log::Level::Warn,None,true){
                println!("Initilaized log at WARN level successfully");
            }
        }
    }

    // Initial MySQL link
    // match CONFIG.get_str("database.url"){
    //     Ok(url) => {
    //         RB.link(&url[..]).await.unwrap();
    //     },
    //     Err(_) => {
    //         return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
    //     }
    // }

    // Initial SSL
    // let mut builder =  SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    // builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    // builder.set_certificate_chain_file("cert.pem").unwrap();

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
    let started_actor = Supervisor::start(|_| casbin_actor);

    let preset_rules = load_csv(walk_csv("policy/conf.d"));
    for mut policy in preset_rules {
        let ptype = policy.remove(0);
        if ptype.starts_with('p') {
            match clone_enforcer.write().await.add_policy(policy).await {
                Ok(_) => info!("Preset policies(p) add successfully"),
                Err(err) => error!("Preset policies(p) add error: {}", err.to_string()),
            };
            continue;
        } else if ptype.starts_with('g') {
            match clone_enforcer
                .write()
                .await
                .add_named_grouping_policy(&ptype, policy)
                .await
            {
                Ok(_) => info!("Preset policies(p) add successfully"),
                Err(err) => error!("Preset policies(g) add error: {}", err.to_string()),
            };
            continue;
        } else {
            unreachable!()
        }
    }


    match CONFIG.get_str("application.server.bind"){
        Ok(url) => {
            HttpServer::new(move || {
                App::new()
                    .data(started_actor.clone())
                    .wrap(
                     Cors::permissive()
                        .allow_any_origin()
                        .allow_any_method()
                        .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                        .allowed_header(header::CONTENT_TYPE)
                        .supports_credentials()
                        .max_age(3600)
                     )
                    .wrap(Logger::default())
                    .wrap(NormalizePath::new(TrailingSlash::Trim))
                    .wrap(casbin_middleware.clone())
                    .wrap(crate::auth::authn::Authentication)
                    .configure(routers::routes)
                    // .wrap(actix_web::middleware::Logger::default())
            })
                .bind(url)?
                .run()
                .await
        },
        Err(_) => {
            return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
        }
    }
}