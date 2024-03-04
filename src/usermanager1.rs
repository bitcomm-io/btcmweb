// // 引入需要的依赖
// use axum::{
//     body::{Bytes, Full},
//     extract::{ContentLengthLimit, Extension, Form, Json, TypedHeader},
//     handler::{get, post},
//     http::{header, StatusCode},
//     response::{Html, IntoResponse, Json as AxumJson},
//     AddExtensionLayer, Router, Server,
// };
// use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
// use serde::{Deserialize, Serialize};
// use sqlx::{postgres::PgPoolOptions, PgPool};
// use std::{convert::Infallible, env, net::SocketAddr, sync::Arc};
// use tower_http::trace::TraceLayer;

// // 定义一个User结构体，用于表示用户数据
// #[derive(Debug, Serialize, Deserialize)]
// struct User {
//     id: i32,
//     username: String,
//     password: String,
//     email: String,
//     token: Option<String>,
// }

// // 定义一个Claims结构体，用于表示jwt的声称
// #[derive(Debug, Serialize, Deserialize)]
// struct Claims {
//     sub: String,
//     exp: usize,
// }

// // 定义一个Register结构体，用于表示用户注册的表单数据
// #[derive(Debug, Deserialize)]
// struct Register {
//     username: String,
//     password: String,
//     email: String,
// }

// // 定义一个Login结构体，用于表示用户登录的表单数据
// #[derive(Debug, Deserialize)]
// struct Login {
//     username: String,
//     password: String,
// }

// // 定义一个Logout结构体，用于表示用户注销的表单数据
// #[derive(Debug, Deserialize)]
// struct Logout {
//     username: String,
// }

// // 定义一个Response结构体，用于表示统一的响应格式
// #[derive(Debug, Serialize)]
// struct Response<T> {
//     code: u16,
//     message: String,
//     data: Option<T>,
// }

// // 定义一个Secret结构体，用于表示jwt的密钥
// #[derive(Debug, Clone)]
// struct Secret(String);

// // 定义一个Pool结构体，用于表示数据库连接池
// #[derive(Debug, Clone)]
// struct Pool(PgPool);

// // 定义一个jwt的编码函数，用于生成token
// fn encode_jwt(user: &User, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
//     let expiration = chrono::Utc::now()
//         .checked_add_signed(chrono::Duration::minutes(15))
//         .expect("valid timestamp")
//         .timestamp();
//     let claims = Claims {
//         sub: user.username.clone(),
//         exp: expiration as usize,
//     };
//     let header = Header::new(jsonwebtoken::Algorithm::HS256);
//     encode(&header, &claims, &EncodingKey::from_secret(secret.as_ref()))
// }

// // 定义一个jwt的解码函数，用于验证token
// fn decode_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
//     decode::<Claims>(
//         &token,
//         &DecodingKey::from_secret(secret.as_ref()),
//         &Validation::new(jsonwebtoken::Algorithm::HS256),
//     )
//     .map(|data| data.claims)
// }

// // 定义一个用户注册的处理函数，用于接收用户注册的表单数据，验证用户名和邮箱是否已存在，如果不存在则将用户数据插入数据库，并返回成功的响应
// async fn register(
//     Extension(pool): Extension<Pool>,
//     Extension(secret): Extension<Secret>,
//     Form(register): Form<Register>,
// ) -> impl IntoResponse {
//     let username = register.username;
//     let password = register.password;
//     let email = register.email;

//     // 检查用户名是否已存在
//     let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
//         .bind(&username)
//         .fetch_optional(&pool.0)
//         .await
//         .unwrap();
//     if let Some(_) = user {
//         return AxumJson(Response {
//             code: 400,
//             message: "用户名已存在".to_string(),
//             data: None,
//         });
//     }

//     // 检查邮箱是否已存在
//     let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
//         .bind(&email)
//         .fetch_optional(&pool.0)
//         .await
//         .unwrap();
//     if let Some(_) = user {
//         return AxumJson(Response {
//             code: 400,
//             message: "邮箱已存在".to_string(),
//             data: None,
//         });
//     }

//     // 将用户数据插入数据库
//     let user = sqlx::query_as::<_, User>(
//         "INSERT INTO users (username, password, email) VALUES ($1, $2, $3) RETURNING *",
//     )
//     .bind(&username)
//     .bind(&password)
//     .bind(&email)
//     .fetch_one(&pool.0)
//     .await
//     .unwrap();

//     // 生成token
//     let token = encode_jwt(&user, &secret.0).unwrap();

//     // 更新用户的token字段
//     let user = sqlx::query_as::<_, User>("UPDATE users SET token = $1 WHERE id = $2 RETURNING *")
//         .bind(&token)
//         .bind(user.id)
//         .fetch_one(&pool.0)
//         .await
//         .unwrap();

//     // 返回成功的响应
//     AxumJson(Response {
//         code: 200,
//         message: "注册成功".to_string(),
//         data: Some(user),
//     })
// }

// // 定义一个用户登录的处理函数，用于接收用户登录的表单数据，验证用户名和密码是否正确，如果正确则生成token并更新用户的token字段，并返回成功的响应
// async fn login(
//     Extension(pool): Extension<Pool>,
//     Extension(secret): Extension<Secret>,
//     Form(login): Form<Login>,
// ) -> impl IntoResponse {
//     let username = login.username;
//     let password = login.password;

//     // 查询用户数据
//     let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
//         .bind(&username)
//         .fetch_optional(&pool.0)
//         .await
//         .unwrap();

//     // 检查用户是否存在
//     if let None = user {
//         return AxumJson(Response {
//             code: 404,
//             message: "用户不存在".to_string(),
//             data: None,
//         });
//     }

//     // 检查密码是否正确
//     let user = user.unwrap();
//     if user.password != password {
//         return AxumJson(Response {
//             code: 401,
//             message: "密码错误".to_string(),
//             data: None,
//         });
//     }

//     // 生成token
//     let token = encode_jwt(&user, &secret.0).unwrap();

//     // 更新用户的token字段
//     let user = sqlx::query_as::<_, User>("UPDATE users SET token = $1 WHERE id = $2 RETURNING *")
//         .bind(&token)
//         .bind(user.id)
//         .fetch_one(&pool.0)
//         .await
//         .unwrap();

//     // 返回成功的响应
//     AxumJson(Response {
//         code: 200,
//         message: "登录成功".to_string(),
//         data: Some(user),
//     })
// }

// // 定义一个用户注销的处理函数，用于接收用户注销的表单数据，验证用户名是否存在，如果存在则清空用户的token字段，并返回成功的响应
// async fn logout(
//     Extension(pool): Extension<Pool>,
//     Form(logout): Form<Logout>,
// ) -> impl IntoResponse {
//     let username = logout.username;

//     // 查询用户数据
//     let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
//         .bind(&username)
//         .fetch_optional(&pool.0)
//         .await
//         .unwrap();

//     // 检查用户是否存在
//     if let None = user {
//         return AxumJson(Response {
//             code: 404,
//             message: "用户不存在".to_string(),
//             data: None,
//         });
//     }

//     // 清空用户的token字段
//     let user = sqlx::query_as::<_, User>("UPDATE users SET token = NULL WHERE username = $1 RETURNING *")
//         .bind(&username)
//         .fetch_one(&pool.0)
//         .await
//         .unwrap();

//     // 返回成功的响应
//     AxumJson(Response {
//         code: 200,
//         message: "注销成功".to_string(),
//         data: Some(user),
//     })
// }
