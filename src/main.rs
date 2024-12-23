use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;

use zero_to_prod::{
    configuration::get_configuration,
    email_client::EmailClient,
    startup::{run, Application},
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Unable to read config file");
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
