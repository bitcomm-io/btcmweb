// 引入需要的依赖
use axum::{
    body::{Bytes, Full},
    extract::{ContentLengthLimit, Extension, Form, Json, TypedHeader},
    handler::{get, post},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json as AxumJson},
    AddExtensionLayer, Router, Server,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use r2d2::{Pool, PooledConnection};
use r2d2_redis::RedisConnectionManager;
use redis_async::{client, resp::RespValue};
use tokio::net::TcpStream;
use std::{convert::Infallible, env, net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;

// 定义一个User结构体，用于表示用户数据
#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    password: String,
    email: String,
    token: Option<String>,
}

// 定义一个Claims结构体，用于表示jwt的声称
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

// 定义一个Register结构体，用于表示用户注册的表单数据
#[derive(Debug, Deserialize)]
struct Register {
    username: String,
    password: String,
    email: String,
}

// 定义一个Login结构体，用于表示用户登录的表单数据
#[derive(Debug, Deserialize)]
struct Login {
    username: String,
    password: String,
}

// 定义一个Logout结构体，用于表示用户注销的表单数据
#[derive(Debug, Deserialize)]
struct Logout {
    username: String,
}

// 定义一个Response结构体，用于表示统一的响应格式
#[derive(Debug, Serialize)]
struct Response<T> {
    code: u16,
    message: String,
    data: Option<T>,
}

// 定义一个Secret结构体，用于表示jwt的密钥
#[derive(Debug, Clone)]
struct Secret(String);

// 定义一个Redis结构体，用于表示redis的连接池
#[derive(Debug, Clone)]
struct Redis(Pool<RedisConnectionManager>);

// 定义一个jwt的编码函数，用于生成token
fn encode_jwt(user: &User, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(15))
        .expect("valid timestamp")
        .timestamp();
    let claims = Claims {
        sub: user.username.clone(),
        exp: expiration as usize,
    };
    let header = Header::new(jsonwebtoken::Algorithm::HS256);
    encode(&header, &claims, &EncodingKey::from_secret(secret.as_ref()))
}

// 定义一个jwt的解码函数，用于验证token
fn decode_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .map(|data| data.claims)
}

// 定义一个用户注册的处理函数，用于接收用户注册的表单数据，验证用户名和邮箱是否已存在，如果不存在则将用户数据插入redis，并返回成功的响应
async fn register(
    Extension(redis): Extension<Redis>,
    Extension(secret): Extension<Secret>,
    Form(register): Form<Register>,
) -> impl IntoResponse {
    let username = register.username;
    let password = register.password;
    let email = register.email;

    // 从连接池中获取一个连接
    let conn = redis.0.get().unwrap();

    // 创建一个异步的redis客户端
    let client = client::paired(&conn);

    // 检查用户名是否已存在
    let exists: bool = client
        .send_and_forget(resp_array!["HEXISTS", "users", &username])
        .await
        .unwrap();
    if exists {
        return AxumJson(Response {
            code: 400,
            message: "用户名已存在".to_string(),
            data: None,
        });
    }

    // 检查邮箱是否已存在
    let exists: bool = client
        .send_and_forget(resp_array!["SISMEMBER", "emails", &email])
        .await
        .unwrap();
    if exists {
        return AxumJson(Response {
            code: 400,
            message: "邮箱已存在".to_string(),
            data: None,
        });
    }

    // 生成一个用户id
    let id: i32 = client
        .send_and_forget(resp_array!["INCR", "user_id"])
        .await
        .unwrap();

    // 将用户数据插入redis
    let user = User {
        id,
        username: username.clone(),
        password: password.clone(),
        email: email.clone(),
        token: None,
    };
    client
        .send_and_forget(resp_array![
            "HSET",
            "users",
            &username,
            bincode::serialize(&user).unwrap()
        ])
        .await
        .unwrap();
    client
        .send_and_forget(resp_array!["SADD", "emails", &email])
        .await
        .unwrap();

    // 生成token
    let token = encode_jwt(&user, &secret.0).unwrap();

    // 更新用户的token字段
    let user = User {
        id,
        username: username.clone(),
        password: password.clone(),
        email: email.clone(),
        token: Some(token.clone()),
    };
    client
        .send_and_forget(resp_array![
            "HSET",
            "users",
            &username,
            bincode::serialize(&user).unwrap()
        ])
        .await
        .unwrap();

    // 返回成功的响应
    AxumJson(Response {
        code: 200,
        message: "注册成功".to_string(),
        data: Some(user),
    })
}

// 定义一个用户登录的处理函数，用于接收用户登录的表单数据，验证用户名和密码是否正确，如果正确则生成token并更新用户的token字段，并返回成功的响应
async fn login(
    Extension(redis): Extension<Redis>,
    Extension(secret): Extension<Secret>,
    Form(login): Form<Login>,
) -> impl IntoResponse {
    let username = login.username;
    let password = login.password;

    // 从连接池中获取一个连接
    let conn = redis.0.get().unwrap();

    // 创建一个异步的redis客户端
    let client = client::paired(&conn);

    // 查询用户数据
    let user: Option<Vec<u8>> = client
        .send_and_forget(resp_array!["HGET", "users", &username])
        .await
        .unwrap();

    // 检查用户是否存在
    if let None = user {
        return AxumJson(Response {
            code: 404,
            message: "用户不存在".to_string(),
            data: None,
        });
    }

    // 检查密码是否正确
    let user: User = bincode::deserialize(&user.unwrap()).unwrap();
    if user.password != password {
        return AxumJson(Response {
            code: 401,
            message: "密码错误".to_string(),
            data: None,
        });
    }

    // 生成token
    let token = encode_jwt(&user, &secret.0).unwrap();

    // 更新用户的token字段
    let user = User {
        id: user.id,
        username: username.clone(),
        password: password.clone(),
        email: user.email.clone(),
        token: Some(token.clone()),
    };
    client
        .send_and_forget(resp_array![
            "HSET",
            "users",
            &username,
            bincode::serialize(&user).unwrap()
        ])
        .await
        .unwrap();

    // 返回成功的响应
    AxumJson(Response {
        code: 200,
        message: "登录成功".to_string(),
        data: Some(user),
    })
}

