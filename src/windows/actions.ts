import { emit } from '@tauri-apps/api/event'
import { getRandomInt } from '@/utils/common';

/**
 * @desc 创建新窗口
 * @param args {object} {label: 'new', url: '/new', width: 500, height: 300, ...}
 */
 export async function createWin(args: unknown) {
  await emit('win-create', args)
}

export async function openPlayerWindow(options: { src: any; sourceId: number }) {
  const { src, sourceId } = options
  if (!src || sourceId === undefined) {
    return
  }
  const url = `/window/player?src=${encodeURIComponent(src)}&sourceId=${sourceId}`
  await createWin({
      label: 'player' + getRandomInt(0, 100),
      title: '播放器',
      url,
      width: 1200,
      height: 900,
  })
}
