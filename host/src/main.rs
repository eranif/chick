use std::net::SocketAddr;
use std::sync::Arc;

mod handlers;
mod models;
mod services;
mod utils;

use crate::handlers::inspect::inspect_handler;
use crate::models::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = oci_client::client::ClientConfig::default();
    let oci_client = Arc::new(oci_client::Client::new(config));
    let app_state = Arc::new(AppState { oci_client });

    let app = axum::Router::new()
        .route("/inspect", axum::routing::post(inspect_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}