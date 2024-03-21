use axum::{
    extract::{ContentLengthLimit, Form},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

// 定义用于JWT的密钥
const SECRET_KEY: &str = "your_secret_key";

#[tokio::main]
async fn main() {
    // 设置路由
    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login));

    // 运行服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// 用户注册处理函数
async fn register(
    ContentLengthLimit(Form(input)): ContentLengthLimit<Form<UserInput>, { 1024 * 10 }>,
) -> impl IntoResponse {
    // 在这里添加用户注册逻辑
    StatusCode::OK
}

// 用户登录处理函数
async fn login(
    ContentLengthLimit(Form(input)): ContentLengthLimit<Form<UserInput>, { 1024 * 10 }>,
) -> impl IntoResponse {
    // 在这里添加用户登录逻辑
    // 成功登录后，创建JWT令牌
    let token = create_jwt(input.username).unwrap();
    (StatusCode::OK, Json(token))
}

// 用户输入数据结构
#[derive(Deserialize)]
struct UserInput {
    username: String,
    password: String,
}

// JWT声明
#[derive(Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}

// 创建JWT令牌的函数
fn create_jwt(username: String) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: username,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET_KEY.as_ref()),
    )
}

// 用户登出处理函数
// 注意：JWT令牌的登出通常是在客户端进行，通过删除或使令牌失效来实现
