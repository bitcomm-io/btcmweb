//! # Bitcomm Web 服务器
//!
//! 该模块定义了 Bitcomm 的 Web 服务器。它使用 Axum 框架处理 HTTP 请求并提供静态文件服务。
//!
//! ## 运行
//!
//! 您可以使用以下命令运行 Web 服务器：
//!
//! ```not_rust
//! cargo run -p example-hello-world
//! ```
//!
//! ## 示例
//!
//! ```rust
//! use bitcommweb::star_webserver;
//!
//! #[tokio::main]
//! async fn main() {
//!     star_webserver().await;
//! }
//! ```
//!
//! ## Web 服务器配置
//!
//! 默认情况下，Web 服务器配置为在 IP 地址 "0.0.0.0" 和端口 "1220" 上监听。您可以通过修改常量 `BITCOMM_ADMINSERVER` 和 `BITCOMM_ADMINSERVER_PORT` 来更改这些值。
//!
//! ## 路由
//!
//! Web 服务器具有以下路由：
//!
//! - `/welcome`：对于 GET 请求，返回 "欢迎来到 Bitcomm！"。
//! - `/jsonrpc`：处理 GET 和 POST 请求。对于 GET，它回复 "Hi Json-RPC from /jsonrpc"。对于 POST，它委托处理给 `jsonrpc::json_rpc_handler` 函数。
//! - `/admin`：从 "admin" 目录提供静态文件服务。如果找不到所请求的资源，它将回退到使用 `ServeFile` 回退机制提供 "index.html"。
//!
//! ## 函数
//!
//! - `get_adminserver_port`：返回格式化后的 Web 服务器的 IP 地址和端口。
//! - `using_serve_dir_with_assets_fallback`：配置并返回带有路由和静态文件服务的 Router，包括对丢失的资源的回退。
//! - `star_webserver`：启动 Web 服务器，绑定到指定的地址并提供配置的路由。使用 Bitcomm 记录器记录服务器地址。
//!

use tracing::info;
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axum::{ routing::{ get, post }, Router };
// use btcmtools::LOGGER;
use serde::{Deserialize, Serialize};
// use slog::info;
use tower_http::{services::{ ServeDir, ServeFile }, trace::TraceLayer};
use crate::jsonrpc;

/// Bitcomm 管理服务器的 IP 地址。
pub static BITCOMM_ADMINSERVER: &str = "0.0.0.0";

/// Bitcomm 管理服务器的端口。
pub static BITCOMM_ADMINSERVER_PORT: &str = "1220";

/// 返回格式化后的 Bitcomm 管理服务器的 IP 地址和端口。
pub fn get_adminserver_port() -> String {
    format!("{}:{}", BITCOMM_ADMINSERVER, BITCOMM_ADMINSERVER_PORT)
}

/// 配置带有路由和静态文件服务的 Router，包括对丢失的资源的回退。
fn using_serve_dir_with_assets_fallback() -> Router {
    // `ServeDir` 允许设置资源未找到时的回退，因此对于 `GET /assets/doesnt-exist.jpg`，它将返回 `index.html` 而不是 404
    let serve_dir = ServeDir::new("admin").not_found_service(ServeFile::new("admin/index.html"));

    Router::new()
        .route(
            "/welcome",
            get(|| async { "欢迎来到 Bitcomm！" })
        )
        .route("/jsonrpc", post(jsonrpc::json_rpc_handler))
        .route(
            "/jsonrpc",
            get(|| async { "你好，来自 /jsonrpc 的 Json-RPC" })
        )
        .route("/register", post(register)).route("/login", post(login))
        .nest_service("/admin", serve_dir.clone())
        .fallback_service(serve_dir)
}
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct User {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
}
async fn register(user: axum::extract::Json<User>) -> axum::response::Json<ApiResponse> {
    // TODO: Implement user registration logic, e.g., store in a database
    println!("Registering user: {:?}", user);
    axum::response::Json(ApiResponse {
        message: "User registered successfully".into(),
    })
}

async fn login(user: axum::extract::Json<User>) -> axum::response::Json<ApiResponse> {
    // TODO: Implement user login logic, e.g., check credentials in a database
    println!("Logging in user: {:?}", user);
    axum::response::Json(ApiResponse {
        message: "User logged in successfully".into(),
    })
}

/// 启动 Bitcomm Web 服务器。绑定到指定的地址并提供配置的路由。使用 Bitcomm 记录器记录服务器地址。
#[allow(unused_variables)]
pub async fn star_webserver() {

    let server_address = get_adminserver_port();
    // 使用路由构建我们的应用程序
    let app = using_serve_dir_with_assets_fallback().layer(TraceLayer::new_for_http());
        // app.layer(TraceLayer::new_for_http());
    // 运行服务器
    let listener = tokio::net::TcpListener::bind(server_address.as_str()).await.unwrap();
    info!("http listening {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}


// //! Run with
// //!
// //! ```not_rust
// //! cargo run -p example-hello-world
// //! ```

// use axum::{ routing::{ get, post }, Router };
// use btcmtools::LOGGER;
// use slog::info;

// use tower_http::services::{ ServeDir, ServeFile };

// use crate::jsonrpc;

// pub static BITCOMM_ADMINSERVER: &str = "0.0.0.0";
// pub static BITCOMM_ADMINSERVER_PORT: &str = "1220";

// ///
// pub fn get_adminserver_port() -> String {
//     format!("{}:{}", BITCOMM_ADMINSERVER, BITCOMM_ADMINSERVER_PORT)
// }

// fn using_serve_dir_with_assets_fallback() -> Router {
//     // `ServeDir` allows setting a fallback if an asset is not found
//     // so with this `GET /assets/doesnt-exist.jpg` will return `index.html`
//     // rather than a 404
//     let serve_dir = ServeDir::new("admin").not_found_service(ServeFile::new("admin/index.html"));

//     Router::new()
//         .route(
//             "/welcome",
//             get(|| async { "Welcom to Bitcomm!" })
//         )
//         .route("/jsonrpc", post(jsonrpc::json_rpc_handler))
//         .route(
//             "/jsonrpc",
//             get(|| async { "Hi Json-RPC from  /jsonrpc" })
//         )
//         .nest_service("/admin", serve_dir.clone())
//         .fallback_service(serve_dir)
// }

// #[allow(unused_variables)]
// pub async fn star_webserver() {
//     let server_address = get_adminserver_port();
//     // build our application with a route
//     let app = using_serve_dir_with_assets_fallback(); //Router::new().route("/", get(handler));
//     // run it
//     let listener = tokio::net::TcpListener::bind(server_address.as_str()).await.unwrap();
//     info!(LOGGER, "http listening on {}", listener.local_addr().unwrap());
//     axum::serve(listener, app).await.unwrap();
// }
