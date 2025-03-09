use reqwest::Client;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex; // 使用 tokio::sync::Mutex
use tokio::time::{sleep, Duration};
use url::Url;

const CACHE_DIR: &str = "./ts_cache";
const CACHE_SIZE: usize = 10; // 缓存的TS数量

use md5;

pub struct TsCache {
    client: Client,
    cache: Arc<Mutex<Vec<(String, u64, u64)>>>,
    m3u8_url: String,
    cache_dir: String, // 添加缓存目录字段
}

impl TsCache {
    pub fn new(m3u8_url: String) -> Self {
        // 使用 m3u8_url 的哈希值作为缓存子目录名
        let hash = format!("{:x}", md5::compute(&m3u8_url));
        let cache_dir = format!("{}/{}", CACHE_DIR, hash);

        fs::create_dir_all(&cache_dir).unwrap();

        Self {
            client: Client::new(),
            cache: Arc::new(Mutex::new(Vec::new())),
            m3u8_url,
            cache_dir,
        }
    }

    pub async fn run(&self) {
        loop {
            match self.fetch_m3u8().await {
                Ok((segments, total_duration, sequence)) => {
                    let cache_clone = Arc::clone(&self.cache);
                    let m3u8_url_clone = self.m3u8_url.clone();
                    let cache_dir_clone = self.cache_dir.clone();
                    // println!("segments: {:?}", segments);
                    // println!("total_duration: {}", total_duration);
                    // println!("sequence: {}", sequence);
                    // println!("m3u8_url_clone: {}", m3u8_url_clone);
                    println!("cache_clone: {:?}", cache_clone);
                    tokio::spawn(async move {
                        Self::manage_cache(
                            &m3u8_url_clone,
                            segments,
                            sequence,
                            cache_clone,
                            &cache_dir_clone,
                        )
                        .await;
                    });
                    let sleep_duration = Duration::from_secs(total_duration / 2);
                    println!("sleep_duration: {}", sleep_duration.as_secs());
                    sleep(sleep_duration).await;
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
        let mut current_duration = 0.0;

        let lines: Vec<&str> = body.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("#EXT-X-MEDIA-SEQUENCE:") {
                sequence = line[22..].parse::<u64>().unwrap_or(0);
            } else if line.starts_with("#EXTINF:") {
                // 正确解析浮点数duration
                if let Some(duration_str) = line.split(':').nth(1) {
                    if let Some(duration_num) =
                        duration_str.trim_end_matches(',').parse::<f64>().ok()
                    {
                        current_duration = duration_num;
                        total_duration += current_duration.ceil() as u64;
                    }
                }

                // 确保还有下一行，且为ts文件
                if i + 1 < lines.len() {
                    let next_line = lines[i + 1];
                    if !next_line.starts_with('#') {
                        segments.push((next_line.to_string(), current_duration.ceil() as u64));
                    }
                }
            }
        }
        Ok((segments, total_duration, sequence))
    }

    async fn manage_cache(
        m3u8_url: &str,
        segments: Vec<(String, u64)>,
        sequence: u64,
        cache: Arc<Mutex<Vec<(String, u64, u64)>>>,
        cache_dir: &str,
    ) {
        let mut cache_lock = cache.lock().await;
        // println!("cache_lock len: {:?}", cache_lock.len());
        let mut index = 0;
        for (segment, duration) in segments {
            let current_segment_sequence = sequence + index;
            // todo 这段其实可以删掉，因为删除了也不会影响到缓存队列的长度，因为删除会在解码后执行。
            // if cache_lock.len() >= CACHE_SIZE {
            //     let (old, _, _) = cache_lock.remove(0);
            //     let old_path = format!("{}/{}", cache_dir, old);
            //     // println!("remove old_path: {}", old_path);
            //     let _ = fs::remove_file(&old_path);
            // }
            if !cache_lock.is_empty() {
                let last_seq = cache_lock.last().map(|(_, s, _)| *s).unwrap_or(0);
                // println!("last_seq: {}", last_seq);
                if current_segment_sequence <= last_seq {
                    index += 1;
                    continue; // 避免重复下载
                }
            }
            let clean_segment = &segment.split('?').next().unwrap_or(&segment);
            let ts_local_path = format!("{}/{}", cache_dir, clean_segment);
            // println!("path: {}", path);
            let ts_url = Self::complete_ts_url(m3u8_url, &segment).unwrap();
            // println!("ts_url: {}", ts_url);
            if let Err(e) = Self::download_ts(&ts_url, &ts_local_path).await {
                eprintln!("Error downloading TS: {e}");
            } else {
                cache_lock.push((clean_segment.to_string(), current_segment_sequence, duration));
            }
            index += 1;
        }
    }

    async fn download_ts(ts_url: &str, ts_local_path: &str) -> Result<(), reqwest::Error> {
        if Path::new(ts_local_path).exists() {
            println!("{} already exists", ts_local_path);
            return Ok(());
        }
        let client = Client::new();
        let response = client.get(ts_url).send().await?.bytes().await?;
        // println!("response: {}", response.len());
        let mut file = File::create(ts_local_path).unwrap();
        file.write_all(&response).unwrap();
        Ok(())
    }

    /// - 返回完整的 TS 文件 URL
    pub fn complete_ts_url(m3u8_url: &str, ts_path: &str) -> Option<String> {
        let base_url = Url::parse(m3u8_url).ok()?.join(".").ok()?; // 解析 base_url
        let full_url = Url::parse(ts_path)
            .or_else(|_| base_url.join(ts_path))
            .ok()?; // 处理相对路径
        Some(full_url.to_string())
    }
    /// 从缓存队列头部获取一个 TS 文件地址
    pub async fn get_next_ts(&self) -> (String, u64, u64) {
        let cache = self.cache.lock().await;
        if !cache.is_empty() {
            let (ts_file, sequence, duration) = &cache[0];
            (format!("{}/{}", self.cache_dir, ts_file), * sequence, * duration)
        } else {
            ("".to_string(), 0, 0)
        }
    }
    /// 删除整个缓存目录
    pub fn clear_cache() -> std::io::Result<()> {
        if Path::new(CACHE_DIR).exists() {
            fs::remove_dir_all(CACHE_DIR)?;
        }
        Ok(())
    }
    /// 删除缓存队列中的第一个TS文件及其对应的缓存文件
    pub async fn remove_first_ts(&self) -> Result<(), std::io::Error> {
        let mut cache = self.cache.lock().await;
        println!("will remove cache: {:?}", cache);
        if let Some((ts_file, _, _)) = cache.first().cloned() {
            // 删除缓存数组中的第一个元素
            cache.remove(0);
            println!("remove ts_file: {}", ts_file);
            // 删除对应的文件
            let file_path = format!("{}/{}", self.cache_dir, ts_file);
            println!("remove file_path: {}", file_path);
            if Path::new(&file_path).exists() {
                fs::remove_file(file_path)?;
            }
            Ok(())
        } else {
            println!("cache is empty");
            Ok(()) // 如果缓存为空，直接返回成功
        }
    }
}
