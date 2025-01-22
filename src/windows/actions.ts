import { emit } from '@tauri-apps/api/event'

/**
 * @desc 创建新窗口
 * @param args {object} {label: 'new', url: '/new', width: 500, height: 300, ...}
 */
 export async function createWin(args) {
  await emit('win-create', args)
}

export async function openPlayerWindow(options) {
  const { src, playlist } = options
  console.log(src, playlist)
  if (!src || !playlist) {
    return
  }
  const url = `/window/player?src=${src}&playlist=${JSON.stringify(playlist)}`
  await createWin({
      label: 'player',
      title: '播放器',
      url,
      width: 1200,
      height: 703,
  })
}
