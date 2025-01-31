<script setup lang="ts">
import { message } from "ant-design-vue";
import { CopyOutlined } from "@ant-design/icons-vue";
import { M3UParser, M3UGroup } from "@/utils/m3u-parser";
import { computed, ref, watch } from "vue";
import { openPlayerWindow } from "@/windows/actions.ts";
import { useRoute } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import Breadcrumb from "@/components/Breadcrumb.vue"; // 引入面包屑组件

// 获取路由对象
const route = useRoute();

// 计算属性：获取动态路由参数 id
const configId = computed(() => route.params.id);
const configName = computed(() => route.query.configName);

// 面包屑数据
const breadcrumbItems = ref([
  { name: "配置列表", path: "/" }, // 返回配置列表页
  { name: `${configName.value}详情`, path: "" }, // 当前页不需要跳转
]);

const playlist = ref<M3UGroup[]>([]);
const activeKey = ref("");

// 从后端获取数据库该m3uUrl的所有视频地址
const getVideoUrls = async (sourceId: any) => {
  console.log("sourceId: ", sourceId);
  const urls: any = await invoke("get_video_urls_command", { sourceId });
  const result = M3UParser.formatChannels(urls);
  console.log("urls: ", result);
  playlist.value = result;
  activeKey.value = result[0].groupName;
};

getVideoUrls(+configId.value);

const play = (src: string) => {
  openPlayerWindow({ src, sourceId: +configId.value });
};

const copyUrl = (url: string) => {
  navigator.clipboard.writeText(url).then(() => {
    message.success("已复制频道地址!");
  });
};
</script>

<template>
  <div class="page-container">
    <Breadcrumb :items="breadcrumbItems" />
    <a-tabs class="tabs" v-model:activeKey="activeKey" type="editable-card" hideAdd>
      <a-tab-pane
        v-for="item in playlist"
        :key="item.groupName"
        :tab="item.groupName"
        :closable="false"
      >
        <a-flex wrap="wrap" gap="small" class="tab-content">
          <div
            class="channel-card"
            v-for="channel in item.channels"
            :key="channel"
            @click="play(channel.url)"
          >
            <div class="channel-logo"><img :src="channel.logo" alt="" /></div>
            <div class="channel-name">{{ channel.name }}</div>
            <div class="channel-actions">
              <a-button type="link" @click.stop="copyUrl(channel.url)">
                <template #icon><CopyOutlined /></template>
              </a-button>
            </div>
          </div>
        </a-flex>
      </a-tab-pane>
    </a-tabs>
  </div>
</template>

<style lang="scss" scoped>
.page-container {
  height: 100vh; /* 设置页面容器高度为视口高度 */
  display: flex;
  flex-direction: column;
  overflow: hidden; /* 防止内容溢出 */
}

.tabs {
  flex: 1; /* 让 tabs 占据剩余空间 */
}

.tab-content {
  max-height: calc(100vh - 200px); /* 设置内容区域的最大高度 */
  overflow-y: auto; /* 允许内容区域滚动 */
  padding: 16px;
}

.ant-tabs-tab {
  border-radius: 32px;
}

.channel-card {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  width: 200px;
  height: 160px;
  background: lightgray;
  border-radius: 8px;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  padding: 16px;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;

  &:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
  }

  .channel-logo {
    > img {
      width: 50px;
    }
  }

  .channel-name {
    font-size: 16px;
    font-weight: 500;
    text-align: center;
    color: #333;
  }

  .channel-actions {
    display: flex;
    justify-content: flex-end;
  }
}
</style>
