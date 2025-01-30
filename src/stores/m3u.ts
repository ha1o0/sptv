import { defineStore } from 'pinia';
import { hashUrl } from '@/utils/md5';
import { M3UGroup } from '@/utils/m3u-parser';

export const useM3uStore = defineStore('m3u', {
  state: () => ({
    sources: {} as Record<string, M3UGroup[]>, // 键是 MD5 哈希值，值是 M3uSource
  }),
  actions: {
    // 添加源
    addSource(url: string, source: M3UGroup[]) {
      const key = hashUrl(url);
      this.sources[key] = source;
    },
    // 删除源
    removeSource(url: string) {
      const key = hashUrl(url);
      delete this.sources[key];
    },
  },
  getters: {
    // 获取源数量
    sourceCount(state): number {
      return Object.keys(state.sources).length;
    },
    // 根据 URL 获取源内容
    getSourceByUrl(state) {
      console.log('state.sources: ', state)
      return (url: string): M3UGroup[] => {
        const key = hashUrl(url);
        console.log('key: ', key)
        return state.sources[key];
      };
    },
    // 根据 MD5 哈希值获取源内容
    getSourceByMd5(state) {
      return (md5: string): M3UGroup[] => state.sources[md5];
    },
  },
});