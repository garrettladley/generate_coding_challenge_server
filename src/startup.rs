use crate::routes::{applicants, challenge, forgot_token, health_check, register, submit};
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/register", web::post().to(register))
            .route("/forgot_token/{nuid}", web::get().to(forgot_token))
            .route("/challenge/{token}", web::get().to(challenge))
            .route("/submit/{token}", web::post().to(submit))
            .route("/applicants", web::get().to(applicants))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
