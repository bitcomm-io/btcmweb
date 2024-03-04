// 引入需要的依赖
use axum::{
    // body::{Bytes, Full},
    extract::{ Extension, Form, Json },
    http::{ header, StatusCode },
    response::{ Html, IntoResponse, Json as AxumJson },
    routing::post,
    Router,
};
use jsonwebtoken::{ decode, encode, DecodingKey, EncodingKey, Header, Validation };
use r2d2::Pool;
use r2d2_redis2::RedisConnectionManager;
use serde::{ Deserialize, Serialize };
use redis::{ Client, Commands };
use std::{ convert::Infallible, env, net::SocketAddr, sync::Arc };
use tower_http::{ add_extension::AddExtensionLayer, trace::TraceLayer };

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

// 定义一个Redis结构体，用于表示redis客户端对象
#[derive(Debug, Clone)]
struct Redis(Client);

// 定义一个jwt的编码函数，用于生成token
fn encode_jwt(user: &User, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc
        ::now()
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
        &Validation::new(jsonwebtoken::Algorithm::HS256)
    ).map(|data| data.claims)
}

// 定义一个用户注册的处理函数，用于接收用户注册的表单数据，验证用户名和邮箱是否已存在，如果不存在则将用户数据插入redis，并返回成功的响应
async fn register(
    Extension(redis): Extension<Redis>,
    Extension(secret): Extension<Secret>,
    Form(register): Form<Register>
) -> impl IntoResponse {
    let username = register.username;
    let password = register.password;
    let email = register.email;

    // 获取redis的连接
    let mut con = redis.0.get_connection().unwrap();

    // 检查用户名是否已存在
    let exists: bool = con.hexists("users", &username).unwrap();
    if exists {
        return AxumJson(Response {
            code: 400,
            message: "用户名已存在".to_string(),
            data: None,
        });
    }

    // 检查邮箱是否已存在
    let exists: bool = con.sismember("emails", &email).unwrap();
    if exists {
        return AxumJson(Response {
            code: 400,
            message: "邮箱已存在".to_string(),
            data: None,
        });
    }

    // 生成一个用户id
    let id: i32 = con.incr("user_id", 1).unwrap();

    // 将用户数据插入redis
    let user = User {
        id,
        username: username.clone(),
        password: password.clone(),
        email: email.clone(),
        token: None,
    };
    con.hset("users", &username, bincode::serialize(&user).unwrap()).unwrap();
    con.sadd("emails", &email).unwrap();

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
    con.hset("users", &username, bincode::serialize(&user).unwrap()).unwrap();

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
    Form(login): Form<Login>
) -> impl IntoResponse {
    let username = login.username;
    let password = login.password;

    // 获取redis的连接
    let mut con = redis.0.get_connection().unwrap();

    // 查询用户数据
    let user: Option<Vec<u8>> = con.hget("users", &username).unwrap();

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
    con.hset("users", &username, bincode::serialize(&user).unwrap()).unwrap();

    // 返回成功的响应
    AxumJson(Response {
        code: 200,
        message: "登录成功".to_string(),
        data: Some(user),
    })
}

// 定义一个用户注销的处理函数，用于接收用户注销的表单数据，验证用户名是否存在，如果存在则清空用户的token字段，并返回成功的响应
async fn logout(
    Extension(redis): Extension<Redis>,
    Form(logout): Form<Logout>
) -> impl IntoResponse {
    let username = logout.username;

    // 获取redis的连接
    let mut con = redis.0.get_connection().unwrap();

    // 查询用户数据
    let user: Option<Vec<u8>> = con.hget("users", &username).unwrap();

    // 检查用户是否存在
    if let None = user {
        return AxumJson(Response {
            code: 404,
            message: "用户不存在".to_string(),
            data: None,
        });
    }

    // 清空用户的token字段
    let user: User = bincode::deserialize(&user.unwrap()).unwrap();
    let user = User {
        id: user.id,
        username: username.clone(),
        password: user.password.clone(),
        email: user.email.clone(),
        token: None,
    };
    con.hset("users", &username, bincode::serialize(&user).unwrap()).unwrap();

    // 返回成功的响应
    AxumJson(Response {
        code: 200,
        message: "注销成功".to_string(),
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
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/register", post(register));

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
