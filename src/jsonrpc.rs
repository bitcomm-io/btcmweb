use axum::{
    extract::Json,
    response::Json as JsonResponse, //routing::post, Router
};
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

// #[tokio::main]
// async fn main() {
//     let server_address = webserver::get_adminserver_port();
//     let app = Router::new().route("/json-rpc", post(json_rpc_handler));

//     let listener = tokio::net::TcpListener::bind(server_address.as_str()).await.unwrap();
//     info!(LOGGER, "http listening on {}", listener.local_addr().unwrap());
//     axum::serve(listener, app).await.unwrap();
// }
