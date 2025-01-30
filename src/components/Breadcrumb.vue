<template>
  <a-breadcrumb class="breadcrumb">
    <a-breadcrumb-item v-for="(item, index) in items" :key="index">
      <router-link v-if="item.path" :to="item.path">{{ item.name }}</router-link>
      <span v-else>{{ item.name }}</span>
    </a-breadcrumb-item>
  </a-breadcrumb>
</template>

<script setup>
// 接收面包屑数据
const props = defineProps({
  items: {
    type: Array,
    required: true,
  },
});
</script>

<style scoped>
.breadcrumb {
  display: flex;
  align-items: center;
  padding: 16px 16px 16px 10px;
}
</style>
