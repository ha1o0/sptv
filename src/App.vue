<script setup lang="ts">
// import { UserOutlined, VideoCameraOutlined, UploadOutlined } from '@ant-design/icons-vue';
import { invoke } from "@tauri-apps/api/core";

import { reactive, watch, h, ref } from 'vue';
import {
  DesktopOutlined,
  AppstoreOutlined,
  VideoCameraOutlined,
  UploadOutlined,
} from '@ant-design/icons-vue';
const state = reactive({
  collapsed: false,
  selectedKeys: ['1'],
  openKeys: ['sub1'],
  preOpenKeys: ['sub1'],
});
const items = reactive([
  {
    key: '1',
    icon: () => h(VideoCameraOutlined),
    label: '首页',
    title: '首页',
  },
  {
    key: '2',
    icon: () => h(DesktopOutlined),
    label: '配置',
    title: '配置',
  },
  {
    key: '3',
    icon: () => h(UploadOutlined),
    label: '收藏',
    title: '收藏',
  },
  {
    key: '4',
    icon: () => h(AppstoreOutlined),
    label: '设置',
    title: '设置',
    children: [
      {
        key: '5',
        label: '通用',
        title: '通用',
      },
      {
        key: '6',
        label: '关于',
        title: '关于',
      },
    ],
  },
]);
watch(
  () => state.openKeys,
  (_val: any, oldVal: string[]) => {
    state.preOpenKeys = oldVal;
  },
);

const onCollapse = (collapsed: boolean, type: string) => {
  console.log(collapsed, type);
};

const onBreakpoint = (broken: boolean) => {
  console.log(broken);
};

const selectedKeys = ref<string[]>(['4']);
const greetMsg = ref("");
const name = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<template>
  <a-layout>
    <a-layout-sider
      breakpoint=""
      collapsed-width="0"
      @collapse="onCollapse"
      @breakpoint="onBreakpoint"
    >
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
      ></a-menu>
    </a-layout-sider>
    <a-layout>
      <a-layout-header :style="{ background: '#fff', padding: 0 }" />
      <a-layout-content :style="{ margin: '5px' }">
        <div class="content">

        </div>
      </a-layout-content>
    </a-layout>
  </a-layout>
</template>

<style scoped>
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

.content {
  padding: 5px;
  height: 100%;
}
</style>

<style>
#app {
  height: 100vh;
}

.ant-layout-sider-children {
  background: white;
}
</style>
