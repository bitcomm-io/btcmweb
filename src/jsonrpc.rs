use axum::{
    extract::{FromRequest, Json}, http::{self, StatusCode}, response::{Json as JsonResponse, Response}, Extension //routing::post, Router
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
// use btcmtools::LOGGER;
use serde::{ Deserialize, Serialize };
// use slog::info;
use std::collections::HashMap;

// use crate::webserver;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRequest {
    jsonrpc: String,
    method: String,
    params: Option<HashMap<String, serde_json::Value>>,
    id: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonResponseWrapper {
    jsonrpc: String,
    result: serde_json::Value,
    id: serde_json::Value,
}

pub async fn json_rpc_handler(req: Json<JsonRequest>) -> JsonResponse<JsonResponseWrapper> {
    let response = match req.method.as_str() {
        "add" => {
            if let Some(params) = &req.params {
                if let (Some(a), Some(b)) = (params.get("a"), params.get("b")) {
                    if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
                        JsonResponseWrapper {
                            jsonrpc: req.jsonrpc.clone(),
                            result: serde_json::to_value(a + b).unwrap(),
                            id: req.id.clone(),
                        }
                    } else {
                        error_response(req.jsonrpc.clone(), req.id.clone())
                    }
                } else {
                    error_response(req.jsonrpc.clone(), req.id.clone())
                }
            } else {
                error_response(req.jsonrpc.clone(), req.id.clone())
            }
        }
        _ => error_response(req.jsonrpc.clone(), req.id.clone()),
    };
    JsonResponse(response)
}

fn error_response(jsonrpc: String, id: serde_json::Value) -> JsonResponseWrapper {
    JsonResponseWrapper {
        jsonrpc,
        result: serde_json::json!({"error": {
            "code": -32601,
            "message": "Method not found",
        }}),
        id,
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    // ...您的声明字段...
}
use axum::extract::request_parts;

async fn jwt_auth_middleware<B>(
    mut req: RequestParts<B>,
) -> Result<RequestParts<B>, Response> {
    if let Some(auth_header) = req.headers().get(http::header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str.trim_start_matches("Bearer ");
                let validation = Validation::new(Algorithm::HS256);
                let key = DecodingKey::from_secret("your-secret-key".as_ref());
                match decode::<Claims>(token, &key, &validation) {
                    Ok(_) => return Ok(req),
                    Err(_) => (),
                }
            }
        }
    }

    Err(Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body("Invalid JWT token".into())
        .unwrap())
}

// #[tokio::main]
// async fn main() {
//     let server_address = webserver::get_adminserver_port();
//     let app = Router::new().route("/json-rpc", post(json_rpc_handler));

//     let listener = tokio::net::TcpListener::bind(server_address.as_str()).await.unwrap();
//     info!(LOGGER, "http listening on {}", listener.local_addr().unwrap());
//     axum::serve(listener, app).await.unwrap();
// }

// // JWT验证中间件
// pub async fn jwt_authenticate_handler(
//     Extension(token): Extension<String>
// ) -> Result<(), JsonResponse<serde_json::Value>> {
//     // let token = req.into_inner().0.token;
//     // 在这里验证 JWT token
//     let secret = "your_secret";
//     let key = DecodingKey::from_secret(secret.as_ref());
//     match decode::<HashMap<String, String>>(&token, &key, &Validation::new(Algorithm::HS256)) {
//         Ok(_) => Ok(()),
//         Err(_) => Err(JsonResponse(serde_json::json!({
//             "error": "Unauthorized"
//         })))
//     }
// }



