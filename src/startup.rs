use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};
use routes::{health_check, subscribe};
use sqlx::{postgres::PgPoolOptions, PgConnection, PgPool};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::{self, DatabaseSettings, Settings},
    email_client::{self, EmailClient},
    routes::{self, confirm},
};

pub struct Application {
    port: u16,
    server: Server,
}


// Wrapper type in order to retreive the URL in subscribe handler
// Retrievel from the context, in actix-web is type-based: using a raw String would expose us to conflicts
pub struct ApplicationBaseUrl(pub String);


pub fn run(
    listner: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server, std::io::Error> {
    // Wrap our TCP Connection in an ARC
    let db_pool = web::Data::new(db_pool);

    // Wrap out Email Connection Pool in an Arc
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    // Make sure we have move to we can capture connection from the surrounding env
    let server = HttpServer::new(move || {
        App::new()
            // Middleware is added using Wrap method
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
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
        let server = run(listener, connection_pool, email_client,configuration.application.base_url)?;
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
