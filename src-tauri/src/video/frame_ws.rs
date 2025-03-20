use axum::extract::ws::{Message, WebSocket};
use flume::Receiver;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct WSFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}

// 定义一个类型别名，使函数签名更清晰
pub type SocketHandler = fn(WebSocket, RouterState) -> Pin<Box<dyn Future<Output = ()> + Send>>;
type RouterState = Arc<tokio::sync::Mutex<Receiver<WSFrame>>>;

pub async fn create_frame_ws(
    frame_rx: Receiver<WSFrame>,
    socket_handler: Option<SocketHandler>,
) -> (u16, mpsc::Sender<()>) {
    use axum::{
        extract::{ws::WebSocketUpgrade, State},
        response::IntoResponse,
        routing::get,
    };
    use tokio::sync::Mutex;

    // 移除了未声明的axum_macros宏，因为它不是必需的
    async fn ws_handler(
        ws: WebSocketUpgrade,
        State(state): State<RouterState>,
        State(socket_handler): State<SocketHandler>,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |socket| socket_handler(socket, state))
    }

    // 默认的 socket 处理函数
    async fn default_handle_socket(mut socket: WebSocket, state: RouterState) {
        let camera_rx = state.lock().await;
        println!("socket connection established");
        let now = std::time::Instant::now();

        loop {
            tokio::select! {
                _ = socket.recv() => {
                    println!("Received message from socket");
                    break;
                },
                incoming_frame = camera_rx.recv_async() => {
                    match incoming_frame {
                        Ok(mut frame) => {
                            frame.data.extend_from_slice(&frame.stride.to_le_bytes());
                            frame.data.extend_from_slice(&frame.height.to_le_bytes());
                            frame.data.extend_from_slice(&frame.width.to_le_bytes());

                            if let Err(e) = socket.send(Message::Binary(frame.data)).await {
                                println!("Failed to send frame to socket: {:?}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            println!(
                                "Connection has been lost! Shutting down websocket server: {:?}",
                                e
                            );
                            break;
                        }
                    }
                }
            }
        }

        let elapsed = now.elapsed();
        println!("Websocket closing after {elapsed:.2?}");
    }

    // 使用提供的处理函数或默认处理函数
    let handler =
        socket_handler.unwrap_or(|socket, state| Box::pin(default_handle_socket(socket, state)));

    let router = axum::Router::new()
        .route(
            "/ws",
            get(move |ws, state| ws_handler(ws, state, axum::extract::State(handler))),
        )
        .with_state(Arc::new(Mutex::new(frame_rx)));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    println!("WebSocket server listening on port {}", port);
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

    tokio::spawn(async move {
        let server = axum::serve(listener, router.into_make_service());
        tokio::select! {
            _ = server => {},
            _ = shutdown_rx.recv() => {
                println!("WebSocket server shutting down");
            }
        }
    });

    (port, shutdown_tx)
}
