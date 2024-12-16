use std::net::TcpListener;

use sqlx::PgPool;
use secrecy::ExposeSecret;

use zero_to_prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subsriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subsriber);

    let config = get_configuration().expect("Unable to read config file");
    let connection_pool = PgPool::connect(&config.database.connection_string().expose_secret())
        .await
        .expect("Could not connect to postgres");
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(&address).expect("Failed to bind to port 8000");
    run(listener, connection_pool)?.await
}
