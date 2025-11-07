use secrecy::ExposeSecret;
use sqlx::{Connection, PgPool};
use std::net::TcpListener;
use zero2prod::configuerations::get_configueration;
use zero2prod::startup::run;
use zero2prod::telemetry::*;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);

    init_subscriber(subscriber);
    let configueration = get_configueration().expect("Failed to read configuerations.");
    let connection = PgPool::connect(&configueration.database.connection_string().expose_secret())
        .await
        .expect("failed to connect to PgDb");
    let address = format!("127.0.0.1:{}", configueration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await
}
