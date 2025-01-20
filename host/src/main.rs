use axum::{
    routing::post,
    Router,
    Json,
};
use hyperlight_common::flatbuffer_wrappers::function_types::{ParameterValue, ReturnType, ReturnValue};
use hyperlight_host::{UninitializedSandbox, MultiUseSandbox, sandbox_state::transition::Noop, sandbox_state::sandbox::EvolvableSandbox};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Deserialize)]
struct EchoRequest {
    message: String,
}

#[derive(Serialize)]
struct EchoResponse {
    result: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {    
    let app = Router::new()
        .route("/echo", post(echo_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn echo_handler(
    Json(payload): Json<EchoRequest>,
) -> Json<EchoResponse> {
    let uninitialized_sandbox = UninitializedSandbox::new(
        hyperlight_host::GuestBinary::FilePath("/usr/local/bin/chick-guest".to_string()),
        None,
        None,
        None,
    ).expect("Failed to create uninitialized sandbox");

    let mut multi_use_sandbox: MultiUseSandbox = uninitialized_sandbox.evolve(Noop::default()).expect("Failed to evolve sandbox");
    
    let result = multi_use_sandbox.call_guest_function_by_name(
        "Echo",
        ReturnType::String,
        Some(vec![ParameterValue::String(payload.message)]),
    );

    match result {
        Ok(ReturnValue::String(value)) => {
            Json(EchoResponse { result: value })
        },
        Ok(_) => {
            Json(EchoResponse { 
                result: "Unexpected return value type from guest function".to_string() 
            })
        },
        Err(e) => {
            Json(EchoResponse { 
                result: format!("Error calling guest function: {:?}", e)
            })
        }
    }
}