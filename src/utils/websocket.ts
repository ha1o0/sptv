import { invoke } from "@tauri-apps/api/core";

class WebSocketClient {
  private socket: WebSocket | null = null;
  private clientId: string | null = null;
  private messageHandlers: Map<string, Function[]> = new Map();
  private binaryType: 'blob' | 'arraybuffer' = 'arraybuffer';  // 直接使用 arraybuffer

  async connect(): Promise<void> {
    try {
      const port = await invoke<number>('get_ws_port');
      const wsUrl = `ws://localhost:${port}/ws`;

      return new Promise((resolve, reject) => {
        this.socket = new WebSocket(wsUrl);
        // 设置为直接接收 ArrayBuffer
        this.socket.binaryType = this.binaryType;

        this.socket.onopen = () => {
          console.log('WebSocket 连接已建立');
          resolve();
        };

        this.socket.onmessage = (event) => {
          try {
            if (event.data instanceof ArrayBuffer) {
                console.log("收到二进制数据, 时间", new Date().toLocaleString() + "." + new Date().getMilliseconds());
                // 直接使用 ArrayBuffer，避免 Blob 转换
                const uint8Array = new Uint8Array(event.data);
                // 这里可以对二进制数据进行处理
                this.handleBinaryMessage(uint8Array);
            } else {
                const message = JSON.parse(event.data);
                if (message.ClientConnected) {
                    this.clientId = message.ClientConnected.client_id;
                }
                this.handleMessage(message);
            }
          } catch (error) {
            console.error('处理消息失败:', error);
          }
        };

        this.socket.onerror = (error) => {
          console.error('WebSocket 错误:', error);
          reject(error);
        };

        this.socket.onclose = () => {
          console.log('WebSocket 连接已关闭');
          this.socket = null;
        };
      });
    } catch (error) {
      console.error('连接 WebSocket 失败:', error);
      throw error;
    }
  }

  // 新增二进制数据处理方法
  private handleBinaryMessage(data: Uint8Array): void {
    if (this.messageHandlers.has('binary')) {
      const handlers = this.messageHandlers.get('binary') || [];
      handlers.forEach(handler => handler(data));
    }
  }

  // 发送消息到服务器
  sendMessage(type: string, data: any): void {
    if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
      console.error('WebSocket 未连接');
      return;
    }

    const message = { [type]: data };
    this.socket.send(JSON.stringify(message));
  }

  // 注册消息处理器
  on(messageType: string, handler: Function): void {
    if (!this.messageHandlers.has(messageType)) {
      this.messageHandlers.set(messageType, []);
    }

    this.messageHandlers.get(messageType)?.push(handler);
  }

  // 处理接收到的消息
  private handleMessage(message: any): void {
    // 遍历消息类型
    for (const type in message) {
      if (this.messageHandlers.has(type)) {
        const handlers = this.messageHandlers.get(type) || [];
        handlers.forEach(handler => handler(message[type]));
      }
    }
  }

  // 关闭连接
  disconnect(): void {
    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }
  }

  // 检查连接状态
  isConnected(): boolean {
    return this.socket !== null && this.socket.readyState === WebSocket.OPEN;
  }
}

// 导出单例
export const wsClient = new WebSocketClient();
