<template>
  <div class="video-container" v-show="currentSrc">
    <div
      class="video-js-box"
      :style="{ width: videoJsBoxWidth }"
      @mouseover="showNavbar = true"
      @mouseleave="showNavbar = false"
    >
      <transition name="fade">
        <div class="video-navbar" v-show="showNavbar">
          <!-- 左边显示当前播放的节目信息 -->
          <div class="navbar-left">{{ currentGroup }} - {{ currentChannelName }}</div>
          <!-- 右边添加一个按钮来控制播放列表的展开和收起 -->
          <div class="navbar-right">
            <button @click="togglePlaylist">
              <AlignRightOutlined />
            </button>
          </div>
        </div>
      </transition>
      <canvas ref="videoCanvas"></canvas>
    </div>

    <!-- 右侧播放列表 -->
    <div
      class="video-playlist-container"
      :style="{ width: playlistWidth }"
      :class="{ collapsed: !isPlaylistVisible }"
    >
      <div class="top">
        <a-select
          v-model:value="currentGroup"
          placeholder="选择分组"
          style="width: 200px"
          :options="groupList"
          @change="handleChangeGroup"
        ></a-select>
      </div>
      <div class="channel-list">
        <div
          class="channel-item"
          :class="{
            'channel-item-active':
              decodeURIComponent(item.url) === decodeURIComponent(currentSrc),
          }"
          v-for="(item, index) in currentChannels"
          :key="item.url + index"
          @click="updateVideoSource(item.url)"
        >
          {{ item.name }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch, computed } from "vue";
import { AlignRightOutlined } from "@ant-design/icons-vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { WebGLRenderer } from "@/utils/webgl-renderer";
import { WebGLYUV2RGBRenderer } from "@/utils/webgl-yuv2rbg-renderer";

const props = defineProps({
  // m3u视频源地址
  src: {
    type: String,
    required: true,
  },
  playlist: {
    type: Array,
    required: true,
  },
  // video.js配置选项
  options: {
    type: Object,
    default: () => ({}),
  },
});

const videoRef = ref(null);
const groupList = ref([]);
const currentGroup = ref(null);
const currentChannels = ref([]);
const currentSrc = ref(props.src);
const currentChannelName = ref(""); // 当前播放的节目名称
const isPlaylistVisible = ref(true); // 控制播放列表的展开和收起
const playlistWidth = ref("230px"); // 播放列表的宽度
const showNavbar = ref(true); // 是否显示播放器顶部导航栏
const lastRenderTime = ref(0);
const renderInterval = ref(40);
const frameList = ref([]);
const videoWidth = ref(1920);
const videoHeight = ref(1080);

const videoCanvas = ref(null);
let webglRenderer = null; // WebGLRenderer 实例
let webGLYUV2RGBRenderer = null;

// 计算 video-js-box 的宽度
const videoJsBoxWidth = computed(() => {
  return isPlaylistVisible.value ? "calc(100% - 230px)" : "100%";
});

// 切换播放列表的展开和收起
const togglePlaylist = () => {
  isPlaylistVisible.value = !isPlaylistVisible.value;
};

// 初始化视频播放器
const initializePlayer = () => {
  if (!props.src) {
    return;
  }
  // webglRenderer = new WebGLRenderer(videoCanvas.value);
  webGLYUV2RGBRenderer = new WebGLYUV2RGBRenderer(videoCanvas.value);
  updatePlaylist(props.src);
};

const renderFn = (timestamp) => {
  if (timestamp - lastRenderTime.value >= renderInterval.value) {
    lastRenderTime.value = timestamp;
    if (frameList.value.length) {
      const frame = frameList.value.shift();
      webGLYUV2RGBRenderer.renderFrame(frame[0], frame[1], frame[2], videoWidth.value, videoHeight.value);
      lastRenderTime.value = timestamp;
    } else {
      console.log("没有帧数据, 空等");
    }
  }
  requestAnimationFrame(renderFn);
}

const fetchFrameData = async (timestamp) => {
  // console.log(
  //   "开始请求帧数据：",
  //   new Date().toLocaleString() + "." + new Date().getMilliseconds()
  // );
  const response = await invoke("test_frame_data2", new Uint8Array([]), {
    headers: {
      url: currentSrc.value,
    },
  });
  // console.log(
  //   "结束请求帧数据：",
  //   new Date().toLocaleString() + "." + new Date().getMilliseconds()
  // );
  // console.log("response: ", response);

  const frameData = new Uint8Array(response);

  // console.log("frameData: ", frameData);

  if (frameData.length === 0) {
    console.log("没有帧数据");
    return;
  }

  // 需要根据帧的尺寸来分割 Y、U、V 平面
  const width = 1920;  // 与 Rust 端 dst_width 一致
  const height = 1080; // 与 Rust 端 dst_height 一致

  // 计算各平面的大小
  const ySize = videoWidth.value * videoHeight.value;
  const uvSize = (videoWidth.value * videoHeight.value) / 4;

  // 分离 YUV 平面
  const yPlane = frameData.slice(0, ySize);
  const uPlane = frameData.slice(ySize, ySize + uvSize);
  const vPlane = frameData.slice(ySize + uvSize, ySize + uvSize * 2);

  // console.log("yPlane: ", yPlane);
  // console.log("uPlane: ", uPlane);
  // console.log("vPlane: ", vPlane);
  frameList.value.push([yPlane, uPlane, vPlane]);
}

// 更新视频源
const updateVideoSource = async (newSrc) => {
  console.log("update src: ", newSrc);
  updatePlaylist(newSrc);
  await invoke("start_video_stream", { url: newSrc });
};

const updatePlaylist = (newSrc) => {
  currentSrc.value = newSrc;
  groupList.value = props.playlist.map((item) => {
    return { value: item.groupName, label: item.groupName };
  });
  props.playlist.forEach((item) => {
    item.channels.forEach((channel) => {
      if (decodeURIComponent(channel.url) === decodeURIComponent(newSrc)) {
        currentGroup.value = item.groupName;
        currentChannels.value = item.channels;
        currentChannelName.value = channel.name; // 更新当前播放的节目名称
      }
    });
  });
};

// 监听src属性变化
watch(
  () => props.src,
  (newSrc) => {
    console.log("updated: ", newSrc);
    updateVideoSource(newSrc);
  }
);

// 组件挂载时初始化播放器
onMounted(() => {
  initializePlayer();
  console.log('current url: ', currentSrc.value)
  listen("video_frame", (event) => {
    console.log("video_frame: ", event.payload);
    const fps = 25;
    const interval = 1000 / fps;
    renderInterval.value = 30;
    setInterval(fetchFrameData, 5);
    setTimeout(() => {
      requestAnimationFrame(renderFn);
    }, 2000);
    // webGLYUV2RGBRenderer.renderFrame(
    //   event.payload[0],
    //   event.payload[1],
    //   event.payload[2],
    //   event.payload[3],
    //   event.payload[4]
    // );
    // webglRenderer.renderFrame(event.payload[0], event.payload[1], event.payload[2]);
  });
});

const handleChangeGroup = (groupName) => {
  props.playlist.forEach((item) => {
    if (item.groupName === groupName) {
      currentChannels.value = item.channels;
    }
  });
};
</script>

<style lang="scss" scoped>
.video-container {
  width: 100%;
  height: 100%;
  display: flex;
  overflow-x: hidden; /* 隐藏横向滚动条 */
  position: relative; /* 确保容器不被压缩 */

  .video-js-box {
    position: relative;
    height: 100%;
    padding: 0;
    transition: width 0.3s ease; /* 添加宽度变化的过渡效果 */
    z-index: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    background: #000; // 添加黑色背景
  }

  canvas {
    position: relative;
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }
  .video-navbar {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 40px;
    padding: 5px 16px;
    z-index: 10;
    background: rgba($color: black, $alpha: 0.8);
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: #f0f0f0;
    .navbar-left {
      font-size: 14px;
    }
    .navbar-right {
      button {
        background: transparent;
        border: none;
        color: #f0f0f0;
        cursor: pointer;
        &:hover {
          color: orangered;
        }
      }
    }
  }
  .video-js {
    width: 100%;
    height: 100%;
    padding: 0;
  }
  .video-playlist-container {
    width: 230px;
    height: 100%; /* 设置播放列表高度为视口高度 */
    position: absolute;
    top: 0;
    right: 0; /* 将播放列表定位到右侧 */
    transform: translateX(0); /* 初始位置 */
    transition: transform 0.3s ease; /* 平移效果 */
    overflow-y: auto; /* 允许播放列表内部滚动 */
    z-index: 5;

    /* 控制播放列表收起时的平移 */
    &.collapsed {
      transform: translateX(100%); /* 向右平移，完全隐藏 */
    }

    .top {
      width: 100%;
      height: 40px;
      background: rgba($color: #000000, $alpha: 0.8);
      padding: 10px;
      color: #f0f0f0;
      display: flex;
      justify-content: flex-start;
      align-items: center;
    }
    .channel-list {
      height: calc(100% - 40px);
      overflow-y: auto;
      color: #f0f0f0;
      background: rgba($color: #000000, $alpha: 0.8);
      .channel-item {
        padding: 10px;
        border-bottom: solid 1px #f0f0f0;
        cursor: pointer;
        &:hover {
          background: rgba($color: #000000, $alpha: 0.5);
        }
        &-active {
          color: orangered;
        }
      }
    }
  }
}
</style>
