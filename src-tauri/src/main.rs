#[tokio::main]
async fn main() {
    // 启动 Tauri 应用（必须在主线程中运行）
    sptv_lib::run().await;
}
