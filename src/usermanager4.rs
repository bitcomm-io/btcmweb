// 引入需要的依赖
use axum::{
    extract::{ Extension, Form}, response::IntoResponse, routing::post, Json, Router
};
#[allow(unused_imports)]
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use r2d2::{Pool, PooledConnection};
use r2d2_redis2::RedisConnectionManager;
#[allow(unused_imports)]
use redis_async::{client, resp::RespValue, resp_array};
// use tokio::net::TcpStream;
#[allow(unused_imports)]
use std::{convert::Infallible, env, net::SocketAddr, sync::Arc};
use tower_http::add_extension::AddExtensionLayer;
// #[macro_use]
// use  redis_async;
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

// 定义一个Login结构体，用于表示用户登录的表单数据
#[derive(Debug, Deserialize)]
struct Login {
    username: String,
    password: String,
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
        return Json(Response {
            code: 404,
            message: "用户不存在".to_string(),
            data: None,
        });
    }

    // 检查密码是否正确
    let user: User = bincode::deserialize(&user.unwrap()).unwrap();
    if user.password != password {
        return Json(Response {
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
    Json(Response {
        code: 200,
        message: "登录成功".to_string(),
        data: Some(user),
    })
}

#[tokio::main]
async fn main() {
    // 初始化日志
    

    // 从环境变量中获取redis的连接字符串和jwt的密钥
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

    // 创建一个redis的连接管理器
    let manager = RedisConnectionManager::new(redis_url).unwrap();

    // 创建一个redis的连接池
    let pool = Pool::builder().build(manager).unwrap();

    // 创建一个redis的连接池对象
    let redis = Redis(pool);

    // 创建一个jwt的密钥对象
    let secret = Secret(jwt_secret);

    // 创建一个路由器，并将登录路由指向login函数
    let app = Router::new()
        // 添加redis和secret作为全局扩展
        .layer(AddExtensionLayer::new(redis))
        .layer(AddExtensionLayer::new(secret))
        // `POST /login` goes to `login`
        .route("/login", post(login));

    // 启动web服务，监听本地的3000端口
    // let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    // println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    // Server::bind(&addr)
        // .serve(app.into_make_service())
        // .await
        // .unwrap();
}
