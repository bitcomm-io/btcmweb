pub mod webserver;
pub mod jsonrpc;
// mod usermanager;
// mod usermanager2;
// mod usermanager3;
// mod usermanager4;


// macro_rules! create_func_map {
//     ($map:ident, $($func:ident),*) => {{
//         $( $map.insert(stringify!($func), $func as fn(Json<JsonRequest>) -> JsonResponse<JsonResponseWrapper>); )*
//     }};
// }

// macro_rules! create_afunc_map {
//     ($map:ident, $($func:ident),*) => {{
//         $( $map.insert(stringify!($func), 
//             std::sync::Arc::new(|request: Json<JsonRequest>| -> JsonResponse<JsonResponseWrapper> {
//                 async move { $func(request).await }
//             }) as std::sync::Arc<dyn Fn(Json<JsonRequest>) -> JsonResponse<JsonResponseWrapper> + Send + Sync>); )*
//     }};
// }

// fn main() {
//     let mut func_map = HashMap::new();
//     create_func_map!(func_map, add, subtract);

//     // 示例：从 HashMap 中调用函数
//     if let Some(func) = func_map.get("add") {
//         let result = func(10, 5);
//         println!("Result of add function: {}", result);
//     }

//     if let Some(func) = func_map.get("subtract") {
//         let result = func(10, 5);
//         println!("Result of subtract function: {}", result);
//     }
// }