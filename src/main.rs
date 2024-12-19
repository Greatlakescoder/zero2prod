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
    let connection_pool = PgPool::connect_lazy_with(config.database.connect_options());
    let address = format!("{}:{}", config.application.host,config.application.port);
    let listener = TcpListener::bind(&address).expect("Failed to bind to port 8000");
    run(listener, connection_pool)?.await
}
