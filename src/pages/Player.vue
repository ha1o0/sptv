<script setup lang="ts">
import { M3UParser, M3UGroup } from "@/utils/m3u-parser";
import { ref } from "vue";
import VideoPlayer from "../components/VideoPlayer.vue"; // 确保路径正确
import { useRouter } from 'vue-router';

const router = useRouter();
const playlist = ref<M3UGroup[]>([]);
const activeKey = ref("");
const currentVideoSrc = ref("");

const play = (url: string) => {
  currentVideoSrc.value = url;
};

const videoOptions = {
  autoplay: true,
  // controls: true,
  // 其他video.js配置选项
};

// 解析结果

const query = router.currentRoute.value.query;
console.log('hello: ', query);
if (query.src) {
  currentVideoSrc.value = decodeURIComponent(query.src) as string;
}
if (query.playlist) {
  playlist.value = JSON.parse(decodeURIComponent(query.playlist) as string);
  activeKey.value = playlist.value[0].groupName;
}
</script>

<template>
  <div>
    <div class="video-container">
      <VideoPlayer :src="currentVideoSrc" :playlist="playlist" :options="videoOptions" />
    </div>
  </div>
</template>

<style lang="scss" scoped>
.ant-tabs-tab {
  border-radius: 32px;
}

.channel-card {
  display: flex;
  width: 200px;
  height: 80px;
  background: lightblue;
  word-break: break-all;
  cursor: pointer;
}

.video-container {
    width: 100%;
    height: 100%;
}
</style>
