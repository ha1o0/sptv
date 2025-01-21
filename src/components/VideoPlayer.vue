<!-- VideoPlayer.vue -->
<template>
  <div class="video-container" v-show="props.src">
    <video
      id="sptv"
      class="video-js"
      preload="auto"
      poster=""
    >
      <source src="" type="video/mp4" />
      <p class="vjs-no-js">
        如果想使用video.js，请确保浏览器可以运行JavaScript，并且支持
        <a href="https://videojs.com/html5-video-support/" target="_blank"
          >HTML5 video</a>
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
        <div v-for="(item, index) in currentChannels" :key="index">
          {{ item.name }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import videojs from 'video.js'
import 'video.js/dist/video-js.css'

const props = defineProps({
  // m3u视频源地址
  src: {
    type: String,
    required: true
  },
  playlist: {
    type: Array,
    required: true
  },
  // video.js配置选项
  options: {
    type: Object,
    default: () => ({})
  }
})

const videoRef = ref(null)
const groupList = ref([])
const currentGroup = ref(null)
const currentChannels = ref([])
let player = null

// 初始化视频播放器
const initializePlayer = () => {
  if (!props.src) {
    return
  }
  // 合并默认配置和用户配置
  const defaultOptions = {
    fluid: true, // 响应式布局
    controls: true,
    autoplay: true,
    preload: 'auto',
    sources: [{
      src: props.src,
      type: 'application/x-mpegURL' // m3u8格式
    }]
  }

  const finalOptions = {
    ...defaultOptions,
    ...props.options
  }

  // 创建播放器实例
  player = videojs('sptv', finalOptions)

  // 监听播放器事件
  player.on('ready', () => {
    console.log('Player is ready')
    player.load()
  })

  player.on('error', (error) => {
    console.error('Video player error:', error)
  })
}

// 更新视频源
const updateVideoSource = (newSrc) => {
  if (!player) {
    initializePlayer()
  } else {
    player.src({
      src: newSrc,
      type: 'application/x-mpegURL'
    })
    console.log('play..')
    player.load()
  }
}

// 监听src属性变化
watch(() => props.src, (newSrc) => {
  console.log('updated: ', newSrc)
  groupList.value = props.playlist.map(item => {
    return { value: item.groupName, label: item.groupName }
  })
  props.playlist.forEach(item => {
    item.channels.forEach(channel => {
      if (channel.url === newSrc) {
        currentGroup.value = item.groupName
        currentChannels.value = item.channels
      }
    })
  })
  updateVideoSource(newSrc)
})

// 组件挂载时初始化播放器
onMounted(() => {
  initializePlayer()
})

// 组件卸载前销毁播放器
onBeforeUnmount(() => {
  if (player) {
    player.dispose()
  }
})

const handleChangeGroup = (groupName) => {
  props.playlist.forEach(item => {
    if (item.groupName === groupName) {
        currentChannels.value = item.channels
      }
  })
}
</script>

<style scoped>
.video-container {
  width: 100%;
  height: 100%;
  display: flex;

  .video-js {
    width: 80%;
    height: 100%;
  }
}
</style>
