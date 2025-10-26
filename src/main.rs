use std::net::TcpListener;
use zero2prod::configuerations::{self, get_configueration};
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configueration = get_configueration().expect("Failed to read configuerations.");

    println!("{:#?}", configueration);
    let address = format!("127.0.0.1:{}", configueration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener)?.await
}
