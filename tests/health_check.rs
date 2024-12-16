use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use zero_to_prod::configuration::{get_configuration, DatabaseSettings};


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
    // Port 0 is OS level, it will trigger an OS scan for avaliable port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}",port);
    let mut configuration = get_configuration().expect("Failed to read config");
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;
    let server = zero_to_prod::startup::run(listener,connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp{
        address: address,
        db_pool: connection_pool
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: "password".to_string(),
        ..config.clone()
    };

    let mut connection = PgConnection::connect(
        &maintenance_settings.connection_string()
    ).await.expect("Failed to connect to postgress");

    connection.execute(format!(r#"CREATE DATABASE "{}";"#,config.database_name).as_str()).await.expect("Failed to create database");

    // Migrate Database
    let connection_pool = PgPool::connect(&config.connection_string()).await.expect("Failed to connect to postgres");
    sqlx::migrate!("./migrations").run(&connection_pool).await.expect("Failed to migrate db");
    connection_pool
    
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let configuration = get_configuration().expect("Failed to get config");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
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
