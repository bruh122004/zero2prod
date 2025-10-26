use sqlx::{query, Connection, PgConnection};
use std::net::TcpListener;
use zero2prod::configuerations::get_configueration;
use zero2prod::startup;

#[tokio::test]

async fn health_check() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &address)) //127.0.0.1:8080/health_check")
        .send()
        .await
        .expect("Failed to request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to randomly bind to port");

    let portnum = listener.local_addr().unwrap().port();

    let server = startup::run(listener).expect("failed to run server");

    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", &portnum)
}

#[tokio::test]
async fn subscribe_returns_OK_code() {
    let app_address = spawn_app();

    let configuration = get_configueration().expect("failed to read configs");

    let connection_string = configuration.database.connection_string();

    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to postgres");

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_all(&mut connection)
        .await
        .expect("query failed!!");
}

#[tokio::test]
async fn subscribe_returns_a_400() {
    let app_address = spawn_app();

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "the API did not failt with bad request {}.",
            error_message
        );
    }
}
