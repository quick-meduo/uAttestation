use actix_web::web;

use crate::api::*;

pub fn routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/api")
              .service(health::echo)
            .service(health::alive)
            .service(token::create)
    );
}