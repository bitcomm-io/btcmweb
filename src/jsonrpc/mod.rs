mod addrpc;

use std::collections::HashMap;
use std::sync::Arc;
use axum::{ extract::Json, response::Json as JsonResponse };
use serde::{ Deserialize, Serialize };
use async_trait::async_trait;
use lazy_static::lazy_static;
use tokio::sync::RwLock;

/// 表示一个 JSON-RPC 请求。
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRequest {
    /// JSON-RPC 协议版本。
    jsonrpc :   String,
    /// JSON-RPC 方法名。
    method  :   String,
    /// JSON-RPC 请求 ID。
    id      :   serde_json::Value,
    // jwt token
    token   :   String,
    /// JSON-RPC 方法参数。
    params  :   Option<HashMap<String, serde_json::Value>>,
}

/// 表示一个 JSON-RPC 响应包装器。
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonResponseWrapper {
    /// JSON-RPC 协议版本。
    jsonrpc: String,
    /// JSON-RPC 响应结果。
    result: serde_json::Value,
    /// JSON-RPC 请求 ID。
    id: serde_json::Value,
}

/// 定义 JSON-RPC 处理器的 trait。
#[async_trait]
pub trait JsonRpcHandle {
    /// 处理 JSON-RPC 请求的函数。
    ///
    /// # 参数
    ///
    /// - `req`: 包含 JSON-RPC 请求数据的 `Json` 结构。
    ///
    /// # 返回
    ///
    /// 返回一个 `JsonResponse` 包含 JSON-RPC 响应数据的 `JsonResponseWrapper` 结构。
    async fn json_rpc_handle(&self, req: Json<JsonRequest>) -> JsonResponse<JsonResponseWrapper>;
}

lazy_static! {
    /// 将 JSON-RPC 方法名与处理函数的映射。
    ///
    /// 该映射存储了 JSON-RPC 方法名与相应处理函数的关联关系。
    static ref JSON_RPC_HANDLE_MAP: RwLock<HashMap<&'static str, Arc<dyn JsonRpcHandle + Send + Sync>>> 
        = {
            let mut m: HashMap<&'static str, Arc<dyn JsonRpcHandle + Send + Sync>> = HashMap::new();
            m.insert("add", Arc::new(addrpc::AddJsonRpcHandler));
            RwLock::new(m)
        };
}

/// 添加 JSON-RPC 处理函数到处理函数映射中。
///
/// # 参数
///
/// - `name`: JSON-RPC 方法名。
/// - `json_rpc_handle`: JSON-RPC 处理函数。
async fn _add_json_rpc_handle(
    name: &'static str,
    json_rpc_handle: Arc<dyn JsonRpcHandle + Send + Sync>
) {
    let mut hashmap = JSON_RPC_HANDLE_MAP.write().await;
    hashmap.insert(name, json_rpc_handle);
}

/// 获取指定名称的 JSON-RPC 处理函数。
///
/// # 参数
///
/// - `name`: JSON-RPC 方法名。
///
/// # 返回
///
/// 返回一个包含 JSON-RPC 处理函数的 `Option`。
async fn get_json_rpc_handle(name: String) -> Option<Arc<dyn JsonRpcHandle + Send + Sync>> {
    let hashmap = JSON_RPC_HANDLE_MAP.read().await;
    hashmap.get(name.as_str()).map(|x| x.clone())
}

/// 调用 JSON-RPC 处理函数。
///
/// 如果未找到指定名称的处理函数，则返回一个错误响应。
///
/// # 参数
///
/// - `req`: 包含 JSON-RPC 请求数据的 `Json` 结构。
///
/// # 返回
///
/// 返回一个 `JsonResponse` 包含 JSON-RPC 响应数据的 `JsonResponseWrapper` 结构。
pub async fn call_json_rpc_handler(req: Json<JsonRequest>) -> JsonResponse<JsonResponseWrapper> {
    match get_json_rpc_handle(req.method.clone()).await {
        Some(func_pointer) => func_pointer.json_rpc_handle(req).await,
        None => error_response(req.jsonrpc.clone(), req.id.clone()),
    }
}

/// 创建一个 JSON-RPC 错误响应。
///
/// # 参数
///
/// - `jsonrpc`: JSON-RPC 协议版本。
/// - `id`: JSON-RPC 请求 ID。
///
/// # 返回
///
/// 返回一个包含错误信息的 `JsonResponseWrapper` 结构。
pub fn error_response(jsonrpc: String, id: serde_json::Value) -> JsonResponse<JsonResponseWrapper> {
    let response = JsonResponseWrapper {
        jsonrpc,
        result: serde_json::Value::Null,
        id,
    };
    JsonResponse(response)
}

// 处理 JSON-RPC 请求的函数。
// ///
// /// 该函数接收一个 JSON-RPC 请求，并根据请求中指定的方法将其分发给相应的处理函数。
// /// 如果找不到指定方法，则返回错误响应。
// ///
// /// # 参数
// ///
// /// - `req`: JSON-RPC 请求。
// ///
// /// # 返回
// ///
// /// JSON-RPC 响应。
// pub async fn _json_rpc_handler(req: Json<JsonRequest>) -> JsonResponse<JsonResponseWrapper> {
//     match JSON_RPC_HANDLE_MAP.get(req.method.as_str()) {
//         Some(func_pointer) => func_pointer(req).await,
//         None => JsonResponse(error_response(req.jsonrpc.clone(), req.id.clone())),
//     }
// }
// //
// lazy_static! {
//     /// 将 JSON-RPC 方法名与处理函数的映射。
//     ///
//     /// 该映射存储了 JSON-RPC 方法名与相应处理函数的关联关系。
//     static ref JSON_RPC_HANDLE_MAP: HashMap<&'static str, Box<dyn Fn(Json<JsonRequest>)
//         -> Pin<Box<dyn Future<Output = JsonResponse<JsonResponseWrapper>>
//             + Send + Sync>> + Send + Sync>>
//         = {
//             let mut m = HashMap::new();
//             m.insert("add", get_async_function_pointer(|json| Box::pin(add(json))));
//             m
//         };
// }
// /// 获取指定 JSON-RPC 方法名的处理函数。
// ///
// /// 该函数从 `JSON_RPC_HANDLE_MAP` 中获取指定 JSON-RPC 方法名对应的处理函数。
// ///
// /// # 参数
// ///
// /// - `handle_name`: JSON-RPC 方法名。
// ///
// /// # 返回
// ///
// /// 处理函数的可选引用。
// fn _get_json_rpc_handle(
//     handle_name: &str
// ) -> Option<
//     &Box<
//         dyn (Fn(
//             JsonResponse<JsonRequest>
//         ) -> Pin<Box<dyn Future<Output = JsonResponse<JsonResponseWrapper>> + Send + Sync>>) +
//             Send +
//             Sync
//     >
// > {
//     JSON_RPC_HANDLE_MAP.get(handle_name)
// }

// /// 为给定的函数生成异步函数指针。
// ///
// /// 该函数接受一个函数 `f`，并返回一个异步函数指针，可用作 JSON-RPC 请求的处理函数。
// ///
// /// # 参数
// ///
// /// - `f`: 要生成异步函数指针的函数。
// ///
// /// # 返回
// ///
// /// 异步函数指针。
// fn get_async_function_pointer<F>(
//     f: F
// ) -> Box<
//         dyn (Fn(
//             Json<JsonRequest>
//         ) -> Pin<Box<dyn Future<Output = JsonResponse<JsonResponseWrapper>> + Send + Sync>>) +
//             Send +
//             Sync
//     >
//     where
//         F: Fn(
//             Json<JsonRequest>
//         ) -> Pin<Box<dyn Future<Output = JsonResponse<JsonResponseWrapper>> + Send + Sync>> +
//             Send +
//             Sync +
//             'static
// {
//     Box::new(move |input| f(input))
// }
