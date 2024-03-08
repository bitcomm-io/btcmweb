use std::{collections::HashMap, error::Error};
use axum::Json;
use redis::{aio::MultiplexedConnection, AsyncCommands, RedisResult};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

use argon2::{self, Config, ThreadMode, Variant, Version};

// 用户结构体
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct User {
    username: String,
    password: String,
}

// 注册用户
pub async fn register_user(
    con: &MultiplexedConnection,
    username: &str,
    password: &str,
) -> RedisResult<()> {
    // 检查用户名是否已存在
    if con.exists(username).await? {
        return Err("Username already exists".into());
    }

    // 创建用户结构体
    let user = User {
        username: username.to_string(),
        password: password.to_string(),
    };

    // 将用户信息存储到 Redis 中
    con.hset(username, "password", &user.password).await?;

    Ok(())
}

// 用户登录
pub async fn login_user(
    con: &MultiplexedConnection,
    username: &str,
    password: &str,
) -> Result<String, Box<dyn Error>> {
    // 获取用户信息
    let stored_password: String = con.hget(username, "password").await?;

    // 检查密码是否匹配
    if password == stored_password {
        // 生成 JWT
        let token = generate_jwt(username)?;

        Ok(token)
    } else {
        Err("Invalid username or password".into())
    }
}

// 用户注销
pub async fn logout_user(
    con: &MultiplexedConnection,
    username: &str,
    token: &str,
) -> RedisResult<()> {
    // 验证 JWT
    if let Ok(decoded) = decode::<HashMap<String, String>>(
        token,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::default(),
    ) {
        // 检查用户名和 JWT 中的用户名是否匹配
        if let Some(decoded_username) = decoded.claims.get("username") {
            if username == decoded_username {
                // 删除用户信息
                con.del(username).await?;
                return Ok(());
            }
        }
    }

    Err("Invalid token or username".into())
}

// 生成 JWT
fn generate_jwt(username: &str) -> Result<String, Box<dyn Error>> {
    let claims = serde_json::to_value(HashMap::from([("username", username)]))?;

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("secret".as_ref()),
    )?;

    Ok(token)
}



// 注册用户
async fn register(json: Json<User>) -> &'static str {
    let con = get_redis_connection().await;

    // 使用 Argon2 进行密码哈希
    let hashed_password = hash_password(&json.password).await;

    match register_user(&con, &json.username, &hashed_password).await {
        Ok(_) => "User registered successfully",
        Err(_) => "Failed to register user",
    }
}

// 用户登录
async fn login(json: Json<User>) -> Result<Json<String>, &'static str> {
    let con = get_redis_connection().await;

    // 获取用户存储的哈希密码
    let stored_hashed_password = get_user_hashed_password(&con, &json.username).await?;

    // 使用 Argon2 验证密码
    if verify_password(&json.password, &stored_hashed_password).await {
        // 生成 JWT Token 或其他会话标识
        let token = generate_token(&json.username);

        Ok(Json(token))
    } else {
        Err("Invalid username or password")
    }
}

// 异步函数：使用 Argon2 对密码进行哈希
async fn hash_password(password: &str) -> String {
    let config = Config::default();
    argon2::hash_raw(password.as_bytes(), b"randomsalt", &config).unwrap()
}

// 异步函数：从 Redis 获取用户的哈希密码
async fn get_user_hashed_password(con: &MultiplexedConnection, username: &str) -> Result<String, RedisError> {
    let user_key = format!("user:{}", username);
    con.hget(&user_key, "password").await
}

// 异步函数：使用 Argon2 验证密码
async fn verify_password(password: &str, hashed_password: &str) -> bool {
    argon2::verify_encoded(hashed_password, password.as_bytes()).unwrap()
}
