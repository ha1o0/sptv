use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct VideoFrame {
    pub y_plane: Vec<u8>,
    pub u_plane: Vec<u8>,
    pub v_plane: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct RingBuffer {
    buffer: VecDeque<VideoFrame>,
    capacity: usize,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, frame: VideoFrame) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front(); // 丢弃最旧的帧
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_ring_buffer() {
        let ring_buffer = Arc::new(Mutex::new(RingBuffer::new(10)));
        
        {
            let mut buffer = ring_buffer.lock().unwrap();
            buffer.push(VideoFrame {
                y_plane: vec![0; 1920 * 1080],
                u_plane: vec![0; 960 * 540],
                v_plane: vec![0; 960 * 540],
                width: 1920,
                height: 1080,
            });
        }
        
        {
            let mut buffer = ring_buffer.lock().unwrap();
            let frames = buffer.pop_n(3);
            assert_eq!(frames.len(), 1);
        }
    }
}
