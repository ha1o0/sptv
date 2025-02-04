<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { reactive, watch, h, ref } from "vue";
import { DesktopOutlined, AppstoreOutlined, UploadOutlined } from "@ant-design/icons-vue";
import { useRouter, useRoute } from "vue-router";

const router = useRouter();
const route = useRoute();

const state = reactive({
  collapsed: false,
  selectedKeys: [route.path],
  openKeys: [""],
  preOpenKeys: [""],
});

const items = reactive([
  {
    key: "/",
    icon: () => h(DesktopOutlined),
    label: "配置",
    title: "配置",
    route: "ConfigList",
  },
  {
    key: "/favorite",
    icon: () => h(UploadOutlined),
    label: "收藏",
    title: "收藏",
    route: "Favorite",
  },
  {
    key: "/setting",
    icon: () => h(AppstoreOutlined),
    label: "设置",
    title: "设置",
    children: [
      {
        key: "/setting/general",
        label: "通用",
        title: "通用",
        route: "Setting",
      },
      {
        key: "/setting/about",
        label: "关于",
        title: "关于",
      },
    ],
  },
]);
watch(
  () => state.openKeys,
  (_val: any, oldVal: string[]) => {
    state.preOpenKeys = oldVal;
  }
);

const onCollapse = (collapsed: boolean, type: string) => {
  console.log(collapsed, type);
};

const handleMenuClick = (item: any) => {
  const routeName = item.item.route;
  if (!routeName) {
    return;
  }
  router.push({
    name: routeName,
  });
};

// 监听路由变化同步 selectedKeys
watch(
  () => route.path,
  (newPath: any) => {
    state.selectedKeys = [newPath];
  }
);

const greetMsg = ref("");
const name = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<template>
  <a-layout>
    <a-layout-sider collapsed-width="0" @collapse="onCollapse">
      <div class="logo">
        <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
      </div>
      <a-menu
        v-model:openKeys="state.openKeys"
        v-model:selectedKeys="state.selectedKeys"
        mode="inline"
        theme="light"
        :inline-collapsed="state.collapsed"
        :items="items"
        @click="handleMenuClick"
      ></a-menu>
    </a-layout-sider>
    <a-layout>
      <a-layout-header :style="{ background: '#fff', padding: 0 }" />
      <a-layout-content>
        <div class="content">
          <router-view v-slot="{ Component }">
            <transition name="slide-y" mode="out-in">
              <component :is="Component" />
            </transition>
          </router-view>
        </div>
      </a-layout-content>
    </a-layout>
  </a-layout>
</template>

<style lang="scss" scoped>
.slide-y-enter-active {
  transition: transform 0.3s ease, opacity 0.3s ease;
}
.slide-y-enter-from,
.slide-y-leave-to {
  transform: translateY(15px);
  opacity: 0;
}
.slide-y-enter-to,
.slide-y-leave-from {
  transform: translateY(0);
  opacity: 1;
}

.logo {
  display: flex;
  align-items: center;
  height: 32px;
  margin: 16px;
  filter: drop-shadow(0 0 2em #747bff);
}

.ant-layout {
  height: 100%;
}

.ant-layout-content {
  background: white;
}

.content {
  padding: 8px;
  width: 100%;
  height: 100%;
  > div {
    width: 100%;
    height: 100%;
  }
}
</style>

<style>
#app {
  height: 100vh;
  overflow-y: hidden;
}

.ant-layout-sider-children {
  background: white;
}
</style>
