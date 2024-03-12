use axum::{
    extract::Json,
    response::Json as JsonResponse
};
use lazy_static::lazy_static;
use serde::{ Deserialize, Serialize };
use std::{ collections::HashMap, future::Future, pin::Pin };

/// 处理 JSON-RPC 请求的函数。
///
/// 该函数接收一个 JSON-RPC 请求，并根据请求中指定的方法将其分发给相应的处理函数。
/// 如果找不到指定方法，则返回错误响应。
///
/// # 参数
///
/// - `req`: JSON-RPC 请求。
///
/// # 返回
///
/// JSON-RPC 响应。
pub async fn json_rpc_handler(req: Json<JsonRequest>) -> JsonResponse<JsonResponseWrapper> {
    match JSON_RPC_HANDLE_MAP.get(req.method.as_str()) {
        Some(func_pointer) => func_pointer(req).await,
        None => JsonResponse(error_response(req.jsonrpc.clone(), req.id.clone())),
    }
}

lazy_static! {
    /// 将 JSON-RPC 方法名与处理函数的映射。
    ///
    /// 该映射存储了 JSON-RPC 方法名与相应处理函数的关联关系。
    static ref JSON_RPC_HANDLE_MAP: HashMap<&'static str, Box<dyn Fn(Json<JsonRequest>) 
        -> Pin<Box<dyn Future<Output = JsonResponse<JsonResponseWrapper>> 
            + Send + Sync>> + Send + Sync>> 
        = {
            let mut m = HashMap::new();
            m.insert("add", get_async_function_pointer(|json| Box::pin(add(json))));
            m
        };
}

/// 获取指定 JSON-RPC 方法名的处理函数。
///
/// 该函数从 `JSON_RPC_HANDLE_MAP` 中获取指定 JSON-RPC 方法名对应的处理函数。
///
/// # 参数
///
/// - `handle_name`: JSON-RPC 方法名。
///
/// # 返回
///
/// 处理函数的可选引用。
fn _get_json_rpc_handle(
    handle_name: &str
) -> Option<
    &Box<
        dyn (Fn(
            JsonResponse<JsonRequest>
        ) -> Pin<Box<dyn Future<Output = JsonResponse<JsonResponseWrapper>> + Send + Sync>>) +
            Send +
            Sync
    >
> {
    JSON_RPC_HANDLE_MAP.get(handle_name)
}

#[derive(Debug, Serialize, Deserialize)]
/// 表示一个 JSON-RPC 请求。
pub struct JsonRequest {
    jsonrpc: String,
    method: String,
    params: Option<HashMap<String, serde_json::Value>>,
    id: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
/// 表示一个 JSON-RPC 响应包装器。
pub struct JsonResponseWrapper {
    jsonrpc: String,
    result: serde_json::Value,
    id: serde_json::Value,
}

/// 为给定的函数生成异步函数指针。
///
/// 该函数接受一个函数 `f`，并返回一个异步函数指针，可用作 JSON-RPC 请求的处理函数。
///
/// # 参数
///
/// - `f`: 要生成异步函数指针的函数。
///
/// # 返回
///
/// 异步函数指针。
fn get_async_function_pointer<F>(
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

/// 处理 "add" JSON-RPC 方法。
///
/// 该函数执行 JSON-RPC 请求中提供的两个数字的加法，并返回结果。
///
/// # 参数
///
/// - `req`: JSON-RPC 请求。
///
/// # 返回
///
/// JSON-RPC 响应。
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

/// 为 JSON-RPC 请求生成错误响应。
///
/// 当请求的方法未找到时，该函数构造 JSON-RPC 请求的错误响应。
///
/// # 参数
///
/// - `jsonrpc`: JSON-RPC 协议版本。
/// - `id`: 请求标识符。
///
/// # 返回
///
/// 表示错误响应的 JSON-RPC 响应包装器。
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
