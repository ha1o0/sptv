<!-- VideoPlayer.vue -->
<template>
  <div class="video-container" v-show="currentSrc">
    <video id="sptv" class="video-js" preload="auto" poster="">
      <source src="" type="video/mp4" />
      <p class="vjs-no-js">
        如果想使用video.js，请确保浏览器可以运行JavaScript，并且支持
        <a href="https://videojs.com/html5-video-support/" target="_blank">HTML5 video</a>
      </p>
    </video>
    <div class="video-playlist-container">
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
          :class="{ 'channel-item-active': item.url === currentSrc }"
          v-for="(item, index) in currentChannels"
          :key="index"
          @click="updateVideoSource(item.url)"
        >
          {{ item.name }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from "vue";
import videojs from "video.js";
import "video.js/dist/video-js.css";

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
let player = null;

// 初始化视频播放器
const initializePlayer = () => {
  if (!props.src) {
    return;
  }
  // 合并默认配置和用户配置
  const defaultOptions = {
    fluid: true, // 响应式布局
    controls: true,
    autoplay: true,
    preload: "auto",
    sources: [
      {
        src: proxyUrl(props.src),
        // type: "application/vnd.apple.mpegurl",
        // type: "application/x-mpegURL", // m3u8格式
        headers: {
          "User-Agent": "Mozilla/5.0 (DirectPlayer)", // 简化 User-Agent
          Referer: "", // 清空 Referer
          "Accept-Language": "en", // 简化语言头
          Cookie: "", // 清空 Cookie
        },
      },
    ],
  };

  const finalOptions = {
    ...defaultOptions,
    ...props.options,
  };

  // 创建播放器实例
  player = videojs("sptv", finalOptions);

  // 监听播放器事件
  player.on("ready", () => {
    console.log("Player is ready", props.playlist);
    updateVideoSource(props.src);
  });

  player.on("error", (error) => {
    console.error("Video player error:", error);
  });
};

// 更新视频源
const updateVideoSource = (newSrc) => {
  if (!player) {
    initializePlayer();
  } else {
    console.log("update src: ", newSrc);
    currentSrc.value = newSrc;
    groupList.value = props.playlist.map((item) => {
      return { value: item.groupName, label: item.groupName };
    });
    console.log("groupList.value: ", groupList.value);
    props.playlist.forEach((item) => {
      item.channels.forEach((channel) => {
        if (channel.url === newSrc) {
          currentGroup.value = item.groupName;
          currentChannels.value = item.channels;
        }
      });
    });
    player.src({
      src: proxyUrl(newSrc),
      // type: "application/x-mpegURL",
    });
    player.load();
  }
};

const proxyUrl = (url) => {
  return url;
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
  .video-js {
    width: 80%;
    height: 100%;
  }
  .video-playlist-container {
    width: 20%;
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
