use warp::{reply::Response, Filter, Rejection};
use warp::hyper::Body;
use reqwest::Client;

#[tokio::main]
async fn main() {
    // 创建一个 reqwest 客户端
    let client = Client::new();

    // 定义一个代理路由
    let proxy = warp::path!("proxy")
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and_then(move |params: std::collections::HashMap<String, String>| {
            let client = client.clone();
            async move {
                if let Some(url) = params.get("url") {
                    // 使用 reqwest 发送请求
                    let resp = client.get(url).send().await.map_err(|_| warp::reject::not_found())?;

                    let status = resp.status();
                        let headers = resp.headers().clone();
                        let body = resp.bytes().await.unwrap_or_default();
                        let mut response = Response::new(Body::from(body));
                        *response.status_mut() = status;
                        *response.headers_mut() = headers;
                    Ok::<_, Rejection>(response)
                } else {
                    // 返回一个 404 错误
                    Err(warp::reject::not_found())
                }
            }
        });

     // 配置 CORS
     let cors = warp::cors()
     .allow_any_origin() // 允许所有域名访问
     .allow_methods(vec!["GET", "POST", "OPTIONS"]) // 允许的 HTTP 方法
     .allow_headers(vec!["Content-Type"]); // 允许的请求头

    // 应用 CORS 配置
    let routes = proxy.with(cors);

    // 启动代理服务器
    tokio::spawn(async {
        warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
    });

    // 启动 Tauri 应用（必须在主线程中运行）
    sptv_lib::run().await;
}
