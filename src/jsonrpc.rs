#[allow(unused_imports)]
use axum::{
    extract::{ FromRequest, Json },
    http::{ self, StatusCode },
    response::{ Json as JsonResponse, Response },
    Extension, //routing::post, Router
};
// use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
// use btcmtools::LOGGER;
use serde::{ Deserialize, Serialize };
// use slog::info;
use std::{ collections::HashMap, future::Future, pin::Pin };

use lazy_static::lazy_static;

lazy_static! {
    static ref FUNCTION_MAP: HashMap<&'static str, Box<dyn Fn(Json<JsonRequest>) 
        -> Pin<Box<dyn Future<Output = JsonResponse<JsonResponseWrapper>> 
            + Send + Sync>> + Send + Sync>> 
        = {
            let mut m = HashMap::new();
            m.insert("add", returns_function_pointer(|json| Box::pin(add(json))));
            // Mutex::new(m)
            m
        };
}

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

fn returns_function_pointer<F>(
    f: F
) -> Box<
        dyn (Fn(
            Json<JsonRequest>
        ) -> Pin<Box<dyn Future<Output = JsonResponse<JsonResponseWrapper>> + Send + Sync>>) +
            Send +
            Sync
    >
    where
        F: Fn(
            Json<JsonRequest>
        ) -> Pin<Box<dyn Future<Output = JsonResponse<JsonResponseWrapper>> + Send + Sync>> +
            Send +
            Sync +
            'static
{
    Box::new(move |input| f(input))
}

pub async fn json_rpc_handler(req: Json<JsonRequest>) -> JsonResponse<JsonResponseWrapper> {
    if let Some(func_pointer) = FUNCTION_MAP.get(req.method.as_str()) {
        func_pointer(req).await
    } else {
        JsonResponse(error_response(req.jsonrpc.clone(), req.id.clone()))
    }
}

async fn add(req: Json<JsonRequest>) -> JsonResponse<JsonResponseWrapper> {
    if let Some(params) = &req.params {
        if let (Some(a), Some(b)) = (params.get("a"), params.get("b")) {
            if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
                return JsonResponse(JsonResponseWrapper {
                    jsonrpc: req.jsonrpc.clone(),
                    result: serde_json::to_value(a + b).unwrap(),
                    id: req.id.clone(),
                });
            }
        }
    }
    JsonResponse(error_response(req.jsonrpc.clone(), req.id.clone()))
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
