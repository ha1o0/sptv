use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::ts_cache::TsCache;

pub struct TsCacheManager {
    caches: Arc<Mutex<HashMap<String, Arc<TsCache>>>>, // 存储多个 TsCache 实例
}

impl TsCacheManager {
    pub fn new() -> Self {
        Self {
            caches: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 获取或创建 `TsCache` 实例
    pub async fn get_or_create_cache(&self, m3u8_url: String) -> Arc<TsCache> {
        let mut caches = self.caches.lock().await;
        if let Some(cache) = caches.get(&m3u8_url) {
            return cache.clone();
        }

        // 如果该 m3u8_url 没有缓存，则创建新的 TsCache
        let cache = Arc::new(TsCache::new(m3u8_url.clone()));
        caches.insert(m3u8_url.clone(), cache.clone());

        // 启动 TsCache 的 `run` 方法
        let cache_clone = cache.clone();
        tokio::spawn(async move {
            cache_clone.run().await;
        });

        cache
    }
}
