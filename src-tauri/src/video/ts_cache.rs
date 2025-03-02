use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tokio::sync::Mutex; // 使用 tokio::sync::Mutex
use tokio::time::{sleep, Duration};
use reqwest::Client;
use url::Url;
use std::sync::Arc;

const CACHE_DIR: &str = "./ts_cache";
const CACHE_SIZE: usize = 10; // 缓存的TS数量

pub struct TsCache {
    client: Client,
    cache: Arc<Mutex<Vec<(String, u64)>>>, // 使用 tokio::sync::Mutex
    m3u8_url: String,
}

impl TsCache {
    pub fn new(m3u8_url: String) -> Self {
        fs::create_dir_all(CACHE_DIR).unwrap();
        Self {
            client: Client::new(),
            cache: Arc::new(Mutex::new(Vec::new())),
            m3u8_url,
        }
    }

    pub async fn run(&self) {
        loop {
            match self.fetch_m3u8().await {
                Ok((segments, total_duration, sequence)) => {
                    let cache_clone = Arc::clone(&self.cache);
                    let m3u8_url_clone = self.m3u8_url.clone();
                    tokio::spawn(async move {
                        Self::manage_cache(&m3u8_url_clone, segments, sequence, cache_clone).await;
                    });
                    sleep(Duration::from_secs(total_duration / 2)).await;
                }
                Err(e) => eprintln!("Error fetching M3U8: {e}"),
            }
        }
    }

    async fn fetch_m3u8(&self) -> Result<(Vec<(String, u64)>, u64, u64), reqwest::Error> {
        let body = self.client.get(&self.m3u8_url).send().await?.text().await?;
        let mut segments = Vec::new();
        let mut total_duration = 0;
        let mut sequence = 0;

        for line in body.lines() {
            if line.starts_with("#EXT-X-MEDIA-SEQUENCE:") {
                sequence = line[22..].parse::<u64>().unwrap_or(0);
            } else if line.starts_with("#EXTINF:") {
                let duration: u64 = line[8..line.len() - 1].parse().unwrap_or(0);
                total_duration += duration;
                if let Some(next_line) = body.lines().skip_while(|l| *l != line).nth(1) {
                    if next_line.ends_with(".ts") {
                        segments.push((next_line.to_string(), duration));
                    }
                }
            }
        }
        Ok((segments, total_duration, sequence))
    }

    async fn manage_cache(m3u8_url: &str, segments: Vec<(String, u64)>, sequence: u64, cache: Arc<Mutex<Vec<(String, u64)>>>) {
        let mut cache_lock = cache.lock().await; // 使用 await 获取锁

        if !cache_lock.is_empty() {
            let last_seq = cache_lock.last().map(|(_, s)| *s).unwrap_or(0);
            if sequence <= last_seq {
                return; // 避免重复下载
            }
        }
        for (segment, duration) in segments {
            // 如果缓存已满，移除第一个元素
            if cache_lock.len() >= CACHE_SIZE {
                // 使用 `if let` 解构 Option
                let (old, _) = cache_lock.remove(0);
                let old_path = format!("{}/{}", CACHE_DIR, old);
                let _ = fs::remove_file(&old_path);
            }
            let ts_url = Self::complete_ts_url(m3u8_url, &segment).unwrap();
            if let Err(e) = Self::download_ts(&ts_url, &segment).await {
                eprintln!("Error downloading TS: {e}");
            } else {
                cache_lock.push((segment.clone(), sequence));
            }
        }
    }

    async fn download_ts(ts_url: &str, segment: &str) -> Result<(), reqwest::Error> {
        let path = format!("{}/{}", CACHE_DIR, segment);
        if Path::new(&path).exists() {
            return Ok(());
        }
        let client = Client::new();
        let response = client.get(ts_url).send().await?.bytes().await?;
        let mut file = File::create(&path).unwrap();
        file.write_all(&response).unwrap();
        Ok(())
    }

    /// - 返回完整的 TS 文件 URL
    pub fn complete_ts_url(m3u8_url: &str, ts_path: &str) -> Option<String> {
        let base_url = Url::parse(m3u8_url).ok()?.join(".").ok()?; // 解析 base_url
        let full_url = Url::parse(ts_path).or_else(|_| base_url.join(ts_path)).ok()?; // 处理相对路径
        Some(full_url.to_string())
    }
}