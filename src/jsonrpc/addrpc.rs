use axum::{ extract::Json, response::Json as JsonResponse };
use async_trait::async_trait;
use super::{error_response, JsonRequest, JsonResponseWrapper, JsonRpcHandle};

/// 表示一个处理 JSON-RPC 请求的具体类型。
pub struct AddJsonRpcHandler;

#[async_trait]
impl JsonRpcHandle for AddJsonRpcHandler {
    /// 处理 JSON-RPC 请求的具体方法。
    ///
    /// 如果请求中包含参数 "a" 和 "b"，则将其相加并返回结果。
    /// 如果参数无效或缺少，则返回错误响应。
    async fn json_rpc_handle(&self, req: Json<JsonRequest>) -> JsonResponse<JsonResponseWrapper> {
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
        // 如果参数无效或缺少，则返回错误响应。
        error_response(req.jsonrpc.clone(), req.id.clone())
    }
}





















// use ctor::ctor;
// use lazy_static::lazy_static;
// lazy_static! {

//     static ref ADD_JSON_RPC_HANDLER :AddJsonRpcHandler = {
//             let add = AddJsonRpcHandler{};
//             crate::jsonrpc::add_json_rpc_handle("add",Arc::new(AddJsonRpcHandler));
//             add
//         };

// }
















// 

// / 处理 "add" JSON-RPC 方法。
// /
// / 该函数执行 JSON-RPC 请求中提供的两个数字的加法，并返回结果。
// /
// / # 参数
// /
// / - `req`: JSON-RPC 请求。
// /
// / # 返回
// /
// JSON-RPC 响应。
// async fn add(req: Json<JsonRequest>) -> JsonResponse<JsonResponseWrapper> {
//     if let Some(params) = &req.params {
//         if let (Some(a), Some(b)) = (params.get("a"), params.get("b")) {
//             if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
//                 return JsonResponse(JsonResponseWrapper {
//                     jsonrpc: req.jsonrpc.clone(),
//                     result: serde_json::to_value(a + b).unwrap(),
//                     id: req.id.clone(),
//                 });
//             }
//         }
//     }
//     JsonResponse(error_response(req.jsonrpc.clone(), req.id.clone()))
// }
