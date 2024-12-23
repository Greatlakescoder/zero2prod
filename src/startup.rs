use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};
use routes::{health_check, subscribe};
use sqlx::{postgres::PgPoolOptions, PgConnection, PgPool};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::{self, DatabaseSettings, Settings},
    email_client::{self, EmailClient},
    routes,
};

pub struct Application {
    port: u16,
    server: Server,
}

pub fn run(
    listner: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    // Wrap our TCP Connection in an ARC
    let db_pool = web::Data::new(db_pool);

    // Wrap out Email Connection Pool in an Arc
    let email_client = web::Data::new(email_client);
    // Make sure we have move to we can capture connection from the surrounding env
    let server = HttpServer::new(move || {
        App::new()
            // Middleware is added using Wrap method
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listner)?
    .run();
    Ok(server)
}
impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid Sender Email");

        let timeout = configuration.email_client.timeout();

        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, connection_pool, email_client)?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(configuration.connect_options())
}
