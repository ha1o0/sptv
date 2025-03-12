use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use super::ts_cache::TsCache;

pub struct TsCacheManager {
    caches: Arc<Mutex<HashMap<String, Arc<TsCache>>>>, // 存储多个 TsCache 实例
}

impl TsCacheManager {
    pub fn new() -> Self {
        // 删除缓存目录
        if let Err(e) = TsCache::clear_cache() {
            eprintln!("Failed to clear cache: {}", e);
        }
        Self {
            caches: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 获取或创建 `TsCache` 实例
    pub async fn get_or_create_cache(&self, m3u8_url: String) -> Arc<TsCache> {
        let mut caches = self.caches.lock().await;
        if let Some(cache) = caches.get(&m3u8_url) {
            // println!("Using existing cache for URL: {}", m3u8_url);
            return cache.clone();
        }

        // 如果该 m3u8_url 没有缓存，则创建新的 TsCache
        // println!("Creating new cache for URL: {}", m3u8_url);
        let cache = Arc::new(TsCache::new(m3u8_url.clone()));
        caches.insert(m3u8_url.clone(), cache.clone());

        // 启动 TsCache 的 `run` 方法
        let cache_clone = cache.clone();
        tokio::spawn(async move {
            cache_clone.run().await;
        });

        // 启动一个监控任务，定期检查缓存状态
        // let cache_monitor = cache.clone();
        // let url_clone = m3u8_url.clone();
        // tokio::spawn(async move {
        //     Self::monitor_cache(cache_monitor, url_clone).await;
        // });

        cache
    }

    /// 监控缓存状态，确保缓存正常工作
    async fn monitor_cache(cache: Arc<TsCache>, url: String) {
        let mut last_cache_size = 0;
        let mut stuck_count = 0;

        loop {
            sleep(Duration::from_secs(5)).await; // 每5秒检查一次

            // 获取当前缓存大小
            let current_size = {
                let cache_lock = cache.get_cache_size().await;
                cache_lock
            };

            println!("Cache monitor - URL: {}, Size: {}", url, current_size);

            // 检测缓存是否停滞
            if current_size == last_cache_size && current_size > 0 {
                stuck_count += 1;
                println!("Cache may be stuck, count: {}", stuck_count);

                // 如果连续3次检测到缓存大小不变，尝试移除第一个TS文件以触发更新
                if stuck_count >= 3 {
                    println!("Attempting to unstuck cache by removing first TS file");
                    if let Err(e) = cache.remove_first_ts().await {
                        eprintln!("Failed to remove first TS: {}", e);
                    }
                    stuck_count = 0; // 重置计数器
                }
            } else {
                stuck_count = 0; // 缓存大小变化，重置计数器
            }

            last_cache_size = current_size;
        }
    }
}
