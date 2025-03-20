use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{interval, Duration};

// 引入 generate_vec 函数
use crate::video::generate_vec;

// 定义 WebSocket 消息类型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WSMessage {
    VideoControl { action: String, url: Option<String> },
    SystemInfo { message: String },
    ClientConnected { client_id: String },
    // 移除 TestData 类型，因为我们将直接发送二进制数据
}

// 定义 WebSocket 状态
pub struct WSState {
    // 使用 broadcast channel 进行消息广播
    tx: broadcast::Sender<WSMessage>,
    // 记录连接的客户端
    clients: Arc<Mutex<HashMap<String, String>>>,
}

impl WSState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tx,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // 广播消息给所有客户端
    pub async fn broadcast_message(&self, message: WSMessage) {
        let _ = self.tx.send(message);
    }
}

// 处理 WebSocket 连接
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<WSState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

// 处理 WebSocket 连接
async fn handle_socket(mut socket: WebSocket, state: Arc<WSState>) {
    // 为新客户端生成唯一ID
    let client_id = uuid::Uuid::new_v4().to_string();

    // 添加客户端到连接列表
    {
        let mut clients = state.clients.lock().await;
        clients.insert(client_id.clone(), "connected".to_string());
        println!("WebSocket 客户端已连接: {}", client_id);
    }

    // 通知客户端已连接
    let connect_msg = WSMessage::ClientConnected {
        client_id: client_id.clone(),
    };
    let _ = socket
        .send(Message::Text(serde_json::to_string(&connect_msg).unwrap()))
        .await;

    // 订阅广播频道
    let mut rx = state.tx.subscribe();

    // 创建一个定时器，每5秒发送一次测试数据
    let mut interval = interval(Duration::from_secs(10));

    loop {
        tokio::select! {
            // 接收来自客户端的消息
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // 处理客户端发送的文本消息
                        match serde_json::from_str::<WSMessage>(&text) {
                            Ok(ws_msg) => {
                                println!("收到客户端消息: {:?}", ws_msg);
                                // 广播消息给所有客户端
                                let _ = state.tx.send(ws_msg);
                            },
                            Err(e) => {
                                println!("解析消息失败: {}", e);
                            }
                        }
                    },
                    Some(Ok(Message::Binary(_))) => {
                        // 处理二进制消息
                        println!("收到二进制消息");
                    },
                    Some(Ok(Message::Close(_))) | None => {
                        // 客户端断开连接
                        println!("客户端断开连接: {}", client_id);
                        let mut clients = state.clients.lock().await;
                        clients.remove(&client_id);
                        break;
                    },
                    Some(Err(e)) => {
                        println!("WebSocket 错误: {}", e);
                        break;
                    },
                    _ => {}
                }
            },

            // 接收广播消息并发送给客户端
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        // 将消息发送给当前客户端
                        if let Err(e) = socket.send(Message::Text(serde_json::to_string(&msg).unwrap())).await {
                            println!("发送消息失败: {}", e);
                            break;
                        }
                    },
                    Err(e) => {
                        println!("接收广播消息失败: {}", e);
                        break;
                    }
                }
            },

            // 定时器触发，发送测试数据
            _ = interval.tick() => {
                println!(
                    "触发定时器，开始发送数据: {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f")
                );
                // 生成测试数据
                let test_data = generate_vec(3_000_000);

                // 直接发送二进制数据
                if let Err(e) = socket.send(Message::Binary(test_data)).await {
                    println!("发送测试数据失败: {}", e);
                    break;
                }
                println!(
                    "测试数据已发送: {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f")
                );
            }
        }
    }
}

// 启动 WebSocket 服务器
pub async fn start_ws_server() -> (u16, Arc<WSState>) {
    let state = Arc::new(WSState::new());
    let app_state = state.clone();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    // 绑定到随机端口
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    println!("WebSocket 服务器启动在端口 {}", port);

    // 启动服务器
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (port, app_state)
}
