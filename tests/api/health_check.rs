use crate::helpers::spawn_app;
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
