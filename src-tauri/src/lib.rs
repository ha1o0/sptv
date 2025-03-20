mod db; // 声明 db 模块
mod proxy; // 声明 proxy 模块
mod video; // 声明 video 模块
mod websocket; // 声明 websocket 模块

use std::sync::Arc;

use video::ts_cache_manager::TsCacheManager;

use crate::db::db::init_db;
use crate::db::video_sources::{
    add_video_source_command, delete_video_source_command, get_video_sources_command,
    update_video_source_command,
};
use crate::db::video_urls::{add_video_urls_command, get_video_urls_command};
use crate::video::{get_video_frame, start_video_stream, test_frame_data, test_frame_data2};
use crate::websocket::{start_ws_server, WSState};

#[derive(Clone)]
struct AppState {
    ts_cache_manager: Arc<TsCacheManager>,
    ws_state: Option<Arc<WSState>>,
    ws_port: Option<u16>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    // 启动代理服务器（在单独的 Tokio 任务中）
    tokio::spawn(async {
        proxy::start_proxy_server().await;
    });

    // 启动 WebSocket 服务器
    let (ws_port, ws_state) = start_ws_server().await;
    println!("WebSocket 服务器运行在端口: {}", ws_port);

    let manager = Arc::new(TsCacheManager::new());

    // 初始化数据库连接池
    let pool = init_db().await.expect("Failed to initialize database");
    ffmpeg_next::init().unwrap();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(pool)
        .manage(AppState {
            ts_cache_manager: manager,
            ws_state: Some(ws_state),
            ws_port: Some(ws_port),
        })
        .invoke_handler(tauri::generate_handler![
            add_video_source_command,
            get_video_sources_command,
            get_video_urls_command,
            delete_video_source_command,
            update_video_source_command,
            add_video_urls_command,
            start_video_stream,
            get_video_frame,
            test_frame_data,
            test_frame_data2,
            get_ws_port,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// 添加一个命令，让前端可以获取 WebSocket 服务器的端口
#[tauri::command]
fn get_ws_port(state: tauri::State<'_, AppState>) -> Result<u16, String> {
    match state.ws_port {
        Some(port) => Ok(port),
        None => Err("WebSocket 服务器未启动".to_string()),
    }
}
