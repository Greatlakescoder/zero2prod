use actix_web::body;
use actix_web::web::get;
use wiremock::matchers::{path,method};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    // %20 is space url encoded, %40 is @
    let body = "name=wes%20hedrick&email=rallycapcoding%40gmail.com";

    Mock::given(path("/email"))
    .and(method("POST"))
    .respond_with(ResponseTemplate::new(200))
    .expect(1)
    .mount(&app.email_server)
    .await;

    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    let app = spawn_app().await;

    // %20 is space url encoded, %40 is @
    let body = "name=wes%20hedrick&email=rallycapcoding%40gmail.com";

    Mock::given(path("/email"))
    .and(method("POST"))
    .respond_with(ResponseTemplate::new(200))
    .expect(1)
    .mount(&app.email_server)
    .await;

    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name, status from subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions");
    assert_eq!(saved.email, "rallycapcoding@gmail.com");
    assert_eq!(saved.name, "wes hedrick");
    assert_eq!(saved.status, "pending_confirmation");
}


#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=wes", "missing the email"),
        ("email=wes", "missing the name"),
        ("", "missing both email and name"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body.into()).await;

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

    let test_cases = vec![
        ("name=&email=wes.h%40gmail.com", "empty_name"),
        ("name=wes&email=", "missing the email"),
        ("name=wes&email=blarg", "not a valid email"),
    ];

    for (body, error_message) in test_cases {
        let response = app.post_subscriptions(body.into()).await;
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
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    //Arrange

    let app = spawn_app().await;
    let body = "name=wes%20h&email=selfishtoaster%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
}



#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    //Arrange

    let app = spawn_app().await;
    let body = "name=wes%20h&email=selfishtoaster%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    
    let confirmation_links = app.get_confirmation_links(&email_request);

    assert_eq!(confirmation_links.html,confirmation_links.plain_text);
}

