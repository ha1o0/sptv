<script setup lang="ts">
import { M3UGroup } from "@/utils/m3u-parser";
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import VideoPlayer from "../components/VideoPlayer.vue"; // 确保路径正确
import { useRouter } from "vue-router";
import { M3UParser } from "@/utils/m3u-parser";

const router = useRouter();
const playlist = ref<M3UGroup[]>([]);
const activeKey = ref("");
const currentVideoSrc = ref("");
const loading = ref(true);

const getVideoUrls = async (sourceId: any) => {
  sourceId = +sourceId;
  console.log("sourceId: ", sourceId);
  const urls: any = await invoke("get_video_urls_command", { sourceId });
  const source = M3UParser.formatChannels(urls);
  console.log("urls: ", source);
  playlist.value = source;
  activeKey.value = playlist.value[0].groupName;
};

const videoOptions = {
  autoplay: true,
  // controls: true,
  // 其他video.js配置选项
};

// 在 onMounted 中执行异步逻辑
onMounted(async () => {
  const query = router.currentRoute.value.query;
  console.log("hello: ", query);
  if (query.src && query.sourceId) {
    await getVideoUrls(query.sourceId);
    currentVideoSrc.value = decodeURIComponent(query.src) as string;
  }
  loading.value = false; // 数据加载完成
});
</script>

<template>
  <div>
    <!-- 显示加载状态 -->
    <div v-if="loading"><a-spin /></div>
    <div v-else class="video-container">
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
  height: 100vh;
}
</style>
