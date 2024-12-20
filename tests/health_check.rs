use secrecy::{ExposeSecret, SecretString};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use std::sync::LazyLock;
use zero_to_prod::{
    configuration::{get_configuration, DatabaseSettings},
    telemetry::{get_subscriber, init_subscriber},
};

// Make sure tracing is only initialized once
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

// tokio::test is the testing equivalent of tokio::main
#[tokio::test]
async fn check_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request");
    assert!(resp.status().is_success());
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    // First time it will be created, otherwise it will be ignnored
    LazyLock::force(&TRACING);

    // Port 0 is OS level, it will trigger an OS scan for avaliable port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut configuration = get_configuration().expect("Failed to read config");
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;
    let server = zero_to_prod::startup::run(listener, connection_pool.clone())
        .expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address: address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: SecretString::new("password".to_string().into()),
        ..config.clone()
    };

    let mut connection = PgConnection::connect_with(&maintenance_settings.connect_options())
        .await
        .expect("Failed to connect to postgress");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate Database
    let connection_pool = PgPool::connect_with(config.connect_options())
        .await
        .expect("Failed to connect to postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate db");
    connection_pool
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let configuration = get_configuration().expect("Failed to get config");

    let mut connection = PgConnection::connect_with(&configuration.database.connect_options())
        .await
        .expect("Failed to connect to postgres");

    let client = reqwest::Client::new();

    // %20 is space url encoded, %40 is @
    let body = "name=wes%20hedrick&email=rallycapcoding%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name from subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscriptions");
    assert_eq!(saved.email, "rallycapcoding@gmail.com");
    assert_eq!(saved.name, "wes hedrick");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=wes", "missing the email"),
        ("email=wes", "missing the name"),
        ("", "missing both email and name"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "Api error {}",
            error_message
        )
    }

    // %20 is space url encoded, %40 is @
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=&email=wes.h%40gmail.com", "empty_name"),
        ("name=wes&email=", "missing the email"),
        ("name=wes&email=blarg", "not a valid email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "Api error {}",
            error_message
        )
    }

    // %20 is space url encoded, %40 is @
}
