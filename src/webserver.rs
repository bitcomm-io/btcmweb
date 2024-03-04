//! Run with
//!
//! ```not_rust
//! cargo run -p example-hello-world
//! ```

use axum::{ routing::get, Router };
use btcmtools::LOGGER;
use slog::info;

use tower_http::services::{ ServeDir, ServeFile };

pub static BITCOMM_ADMINSERVER     :&str = "0.0.0.0";
pub static BITCOMM_ADMINSERVER_PORT:&str = "1220"; 

/// 
fn get_adminserver_port() -> String {
    format!("{}:{}", BITCOMM_ADMINSERVER, BITCOMM_ADMINSERVER_PORT)
}

fn using_serve_dir_with_assets_fallback() -> Router {
    // `ServeDir` allows setting a fallback if an asset is not found
    // so with this `GET /assets/doesnt-exist.jpg` will return `index.html`
    // rather than a 404
    let serve_dir = ServeDir::new("admin").not_found_service(ServeFile::new("admin/index.html"));

    Router::new()
        .route(
            "/foo",
            get(|| async { "Hi from /foo" })
        )
        .nest_service("/admin", serve_dir.clone())
        .fallback_service(serve_dir)
}

#[allow(unused_variables)]
pub async fn star_webserver() {
    let server_address = get_adminserver_port();
    // build our application with a route
    let app = using_serve_dir_with_assets_fallback(); //Router::new().route("/", get(handler));
    // run it
    let listener = tokio::net::TcpListener::bind(server_address.as_str()).await.unwrap();
    info!(LOGGER, "http listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
 