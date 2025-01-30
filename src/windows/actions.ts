import { emit } from '@tauri-apps/api/event'

/**
 * @desc 创建新窗口
 * @param args {object} {label: 'new', url: '/new', width: 500, height: 300, ...}
 */
 export async function createWin(args: unknown) {
  await emit('win-create', args)
}

export async function openPlayerWindow(options: { src: any; m3uSourceUrl: string }) {
  const { src, m3uSourceUrl } = options
  console.log(src, m3uSourceUrl)
  if (!src || !m3uSourceUrl) {
    return
  }
  const url = `/window/player?src=${src}&m3uSourceUrl=${m3uSourceUrl}`
  await createWin({
      label: 'player',
      title: '播放器',
      url,
      width: 1200,
      height: 703,
  })
}
