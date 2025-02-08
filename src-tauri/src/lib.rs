mod db; // 声明 db 模块
mod proxy; // 声明 proxy 模块

use crate::db::db::init_db;
use crate::db::video_sources::{
    add_video_source_command, delete_video_source_command, get_video_sources_command,
    update_video_source_command,
};
use crate::db::video_urls::{add_video_urls_command, get_video_urls_command};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    // 启动代理服务器（在单独的 Tokio 任务中）
    tokio::spawn(async {
        proxy::start_proxy_server().await;
    });

    // 初始化数据库连接池
    let pool = init_db().await.expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(pool)
        .invoke_handler(tauri::generate_handler![
            add_video_source_command,
            get_video_sources_command,
            get_video_urls_command,
            delete_video_source_command,
            update_video_source_command,
            add_video_urls_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
