use std::collections::{VecDeque, HashMap};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct VideoFrame {
    pub y_plane: Vec<u8>,
    pub u_plane: Vec<u8>,
    pub v_plane: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

// 将原来的 RingBuffer 重命名为 FrameBuffer
pub struct FrameBuffer {
    buffer: VecDeque<VideoFrame>,
    capacity: usize,
}

impl FrameBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, frame: VideoFrame) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(frame);
    }

    pub fn pop_n(&mut self, n: usize) -> Vec<VideoFrame> {
        let mut frames = Vec::new();
        for _ in 0..n.min(self.buffer.len()) {
            if let Some(frame) = self.buffer.pop_front() {
                frames.push(frame);
            }
        }
        frames
    }
}

// 新增 RingBufferManager 来管理多个视频源的帧缓存
pub struct RingBufferManager {
    buffers: HashMap<String, FrameBuffer>,
    default_capacity: usize,
}

impl RingBufferManager {
    pub fn new(default_capacity: usize) -> Self {
        Self {
            buffers: HashMap::new(),
            default_capacity,
        }
    }
    pub fn get_buffer_size(&self, url: &str) -> usize {
        if let Some(buffer) = self.buffers.get(url) {
            buffer.buffer.len()
        } else {
            0
        }
    }
    pub fn push(&mut self, url: &str, frame: VideoFrame) {
        self.buffers
            .entry(url.to_string())
            .or_insert_with(|| FrameBuffer::new(self.default_capacity))
            .push(frame);
    }

    // 新增方法：使用指定容量创建缓冲区
    pub fn create_buffer(&mut self, url: &str, capacity: usize) {
        self.buffers.insert(url.to_string(), FrameBuffer::new(capacity));
    }

    // 修改 push 方法，增加可选的 capacity 参数
    pub fn push_with_capacity(&mut self, url: &str, frame: VideoFrame, capacity: Option<usize>) {
        if let Some(cap) = capacity {
            self.buffers
                .entry(url.to_string())
                .or_insert_with(|| FrameBuffer::new(cap))
                .push(frame);
        } else {
            self.push(url, frame);
        }
    }

    pub fn pop_n(&mut self, url: &str, n: usize) -> Vec<VideoFrame> {
        if let Some(buffer) = self.buffers.get_mut(url) {
            buffer.pop_n(n)
        } else {
            Vec::new()
        }
    }

    pub fn remove_buffer(&mut self, url: &str) -> Option<FrameBuffer> {
        self.buffers.remove(url)
    }
}
