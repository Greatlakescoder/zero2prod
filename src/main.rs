use std::net::TcpListener;

use sqlx::{Connection, PgConnection, PgPool};
use zero_to_prod::{
    configuration::get_configuration,
    startup::run,
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = get_configuration().expect("Unable to read config file");
    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Could not connect to postgres");
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(&address).expect("Failed to bind to port 8000");
    run(listener, connection_pool)?.await
}
