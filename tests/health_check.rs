use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{query, Connection, PgConnection, PgPool};
use std::net::TcpListener;
use zero2prod::configuerations::{get_configueration, DatabaseSettings};
use zero2prod::startup;
use zero2prod::telemetry::*;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_severity_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_severity_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_severity_level, std::io::sink);
        init_subscriber(subscriber);
    }
});
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}
#[tokio::test]
async fn health_check() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &address.await.address)) //127.0.0.1:8080/health_check")
        .send()
        .await
        .expect("Failed to request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to randomly bind to port");

    let portnum = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{portnum}");
    let mut configuration = get_configueration().expect("failed to read configs");
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();
    let connection_pool = configure_db(&configuration.database).await;

    let server = startup::run(listener, connection_pool.clone()).expect("failed to run server");

    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_db(config: &DatabaseSettings) -> PgPool {
    let mut connection =
        PgConnection::connect(&config.connection_string_without_db().expose_secret())
            .await
            .expect("failed to connect to Postgres");
    sqlx::query(&format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .execute(&mut connection)
        .await
        .expect("faild to create database");

    let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

#[tokio::test]
async fn subscribe_returns_OK_code() {
    let app_address = spawn_app().await;

    let configuration = get_configueration().expect("failed to read configs");

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursuladiff_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_address.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app_address.db_pool)
        .await
        .expect("query failed!!");
}

#[tokio::test]
async fn subscribe_returns_a_400() {
    let app_address = spawn_app().await;

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app_address.address))
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
