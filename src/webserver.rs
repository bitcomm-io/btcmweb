//! Run with
//!
//! ```not_rust
//! cargo run -p example-hello-world
//! ```

use std::sync::Arc;

use axum::{response::Html, routing::get, Router};
use btcmnetwork::connservice::ClientPoolManager;

#[allow(unused_variables)]
pub async fn star_webserver(cpm0:Arc<tokio::sync::Mutex<ClientPoolManager>>) {
    // build our application with a route
    let app = Router::new().route("/", get(handler));
    // run it
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1220").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Welcome to Bitcomm World!</h1>")
}