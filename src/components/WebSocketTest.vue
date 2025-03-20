<template>
  <div class="websocket-test">
    <h2>WebSocket 测试</h2>
    <div class="connection-controls">
      <button @click="connectWebSocket" :disabled="isConnected">连接 WebSocket</button>
      <button @click="disconnectWebSocket" :disabled="!isConnected">断开 WebSocket</button>
    </div>

    <div v-if="isConnected" class="status connected">
      已连接 - 客户端 ID: {{ clientId }}
    </div>
    <div v-else class="status disconnected">
      未连接
    </div>

    <div class="data-display">
      <h3>接收到的测试数据</h3>
      <div v-if="testData.length > 0">
        <p>数据长度: {{ testData.length }}</p>
        <p>数据预览 (前10个元素): {{ testData.slice(0, 10).join(', ') }}</p>
      </div>
      <div v-else>
        <p>暂无数据</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { wsClient } from '../utils/websocket';

const isConnected = ref(false);
const clientId = ref('');
const testData = ref<number[]>([]);

// 连接 WebSocket
async function connectWebSocket() {
  try {
    await wsClient.connect();
    isConnected.value = wsClient.isConnected();
  } catch (error) {
    console.error('连接 WebSocket 失败:', error);
  }
}

// 断开 WebSocket 连接
function disconnectWebSocket() {
  wsClient.disconnect();
  isConnected.value = false;
  clientId.value = '';
}

// 注册消息处理器
onMounted(() => {
  // 处理客户端连接消息
  wsClient.on('ClientConnected', (data: any) => {
    clientId.value = data.client_id;
  });

  // 处理测试数据消息
  wsClient.on('TestData', (data: any) => {
    testData.value = data.data;
  });

  // 自动连接 WebSocket
  connectWebSocket();
});

// 组件卸载时断开连接
onUnmounted(() => {
  disconnectWebSocket();
});
</script>

<style scoped>
.websocket-test {
  padding: 20px;
  border: 1px solid #ccc;
  border-radius: 5px;
  margin: 20px;
}

.connection-controls {
  margin-bottom: 15px;
}

button {
  margin-right: 10px;
  padding: 8px 16px;
  background-color: #4CAF50;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

button:disabled {
  background-color: #cccccc;
  cursor: not-allowed;
}

.status {
  padding: 10px;
  margin-bottom: 15px;
  border-radius: 4px;
}

.connected {
  background-color: #dff0d8;
  color: #3c763d;
}

.disconnected {
  background-color: #f2dede;
  color: #a94442;
}

.data-display {
  background-color: #f5f5f5;
  padding: 15px;
  border-radius: 4px;
}
</style>
