mod proxy;

#[tokio::main]
async fn main() {
    // 启动代理服务器（在单独的 Tokio 任务中）
    tokio::spawn(async {
        proxy::start_proxy_server().await;
    });

    // 启动 Tauri 应用（必须在主线程中运行）
    sptv_lib::run().await;
}
