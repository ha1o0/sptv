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
      <!-- <video id="sptv" class="video-js" preload="auto" poster="">
        <source src="" type="video/mp4" />
        <p class="vjs-no-js">
          如果想使用video.js，请确保浏览器可以运行JavaScript，并且支持
          <a href="https://videojs.com/html5-video-support/" target="_blank"
            >HTML5 video</a
          >
        </p>
      </video> -->
    </div>

    <!-- 右侧播放列表 -->
    <div class="video-playlist-container" :style="{ width: playlistWidth }">
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
// import videojs from "video.js";
// import "video.js/dist/video-js.css";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

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

const videoCanvas = ref(null);
let gl, texture;
let player = null;

// 计算 video-js-box 的宽度
const videoJsBoxWidth = computed(() => {
  return isPlaylistVisible.value ? "calc(100% - 230px)" : "100%";
});

// 切换播放列表的展开和收起
const togglePlaylist = () => {
  isPlaylistVisible.value = !isPlaylistVisible.value;
  playlistWidth.value = isPlaylistVisible.value ? "230px" : "0";
};

const initWebGL = () => {
  const canvas = videoCanvas.value;
  canvas.width = 1000; // 你可以根据视频尺寸修改
  canvas.height = 720;

  gl = canvas.getContext("webgl");
  if (!gl) {
    console.error("WebGL 不受支持");
    return;
  }

  // 创建 WebGL 纹理
  texture = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
}

const updateFrame = ([width, height, data]) => {
  console.log('update frame: ', width, height, data)
  if (!gl) return;

  const canvas = videoCanvas.value;
  canvas.width = width;
  canvas.height = height;

  // 更新 WebGL 纹理
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, width, height, 0, gl.RGB, gl.UNSIGNED_BYTE, new Uint8Array(data));

  // 渲染 WebGL 纹理
  gl.clear(gl.COLOR_BUFFER_BIT);
  gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
}

// 初始化视频播放器
const initializePlayer = () => {
  if (!props.src) {
    return;
  }
  // // 合并默认配置和用户配置
  // const defaultOptions = {
  //   fluid: true, // 响应式布局
  //   controls: true,
  //   autoplay: true,
  //   preload: "auto",
  //   sources: [
  //     {
  //       src: props.src,
  //     },
  //   ],
  // };

  // const finalOptions = {
  //   ...defaultOptions,
  //   ...props.options,
  // };

  initWebGL();

  updatePlaylist(props.src);
};

// 更新视频源
const updateVideoSource = (newSrc) => {
  invoke("start_video_stream", { url: newSrc });
  console.log("update src: ", newSrc);
  updatePlaylist(newSrc);
  // if (!player) {
  //   initializePlayer();
  // } else {
  //   console.log("update src: ", newSrc);
  //   updatePlaylist(newSrc);
  //   player.src({
  //     src: newSrc,
  //   });
  //   player.load();
  // }
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
  listen("video_frame", (event) => updateFrame(event.payload));
});

// 组件卸载前销毁播放器
onBeforeUnmount(() => {
  if (player) {
    player.dispose();
  }
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
  .video-js-box {
    position: relative;
    height: 100%;
    padding: 0;
    transition: width 0.3s ease; /* 添加宽度变化的过渡效果 */
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
    width: v-bind(playlistWidth); /* 动态绑定宽度 */
    height: 100%; /* 设置播放列表高度为视口高度 */
    overflow-y: auto; /* 允许播放列表内部滚动 */
    transition: width 0.3s ease; /* 添加宽度变化的过渡效果 */
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
