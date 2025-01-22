<script setup lang="ts">
import Windows from "./windows";

const windowsInstance = new Windows()
windowsInstance.listen()

</script>

<template>
  <div id="app">
    <router-view />
  </div>
</template>

<style>
#app {
  height: 100vh;
}

.ant-layout-sider-children {
  background: white;
}
</style>
