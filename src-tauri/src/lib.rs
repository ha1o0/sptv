mod db; // 声明 db 模块

use crate::db::db::init_db;
use crate::db::video_sources::{add_video_source_command, get_video_sources_command, update_video_source_command, delete_video_source_command};
use crate::db::video_urls::{get_video_urls_command, add_video_urls_command};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    // 初始化数据库连接池
    let pool = init_db().await.expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(pool)
        .invoke_handler(tauri::generate_handler![
            greet,
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