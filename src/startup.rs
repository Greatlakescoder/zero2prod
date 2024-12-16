use tracing_actix_web::TracingLogger;
use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};
use routes::{health_check, subscribe};
use sqlx::{PgConnection, PgPool};
use std::net::TcpListener;

use crate::routes;

pub fn run(listner: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap our TCP Connection in an ARC
    let db_pool = web::Data::new(db_pool);
    // Make sure we have move to we can capture connection from the surrounding env
    let server = HttpServer::new(move || {
        App::new()
            // Middleware is added using Wrap method
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
    })
    .listen(listner)?
    .run();
    Ok(server)
}
