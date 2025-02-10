use ffmpeg_next::{codec, format, frame, software::scaling, util::format::Pixel};
use std::thread;
use tauri::{AppHandle, Emitter};

struct VideoStreamer {
    app_handle: AppHandle,
}

impl VideoStreamer {
    pub fn new(app_handle: AppHandle) -> Self {
        VideoStreamer { app_handle }
    }

    pub fn start_stream(&self, url: String) {
        let app_handle = self.app_handle.clone();
        thread::spawn(move || {
            ffmpeg_next::init().unwrap();
    
            let mut ictx = format::input(&url).expect("无法打开 M3U8 流");
            let stream = ictx.streams().best(ffmpeg_next::media::Type::Video).expect("没有找到视频流");
            let stream_index = stream.index();
    
            let mut decoder = codec::Context::from_parameters(stream.parameters())
                .unwrap()
                .decoder()
                .video()
                .unwrap();
    
            let decoder_width = decoder.width();
            let decoder_height = decoder.height();
            let mut scaler = scaling::Context::get(
                decoder.format(),
                decoder_width,
                decoder_height,
                Pixel::RGB24, // 使用更轻量的格式
                320, 320 * decoder_height / decoder_width,  // 降低分辨率
                scaling::Flags::BILINEAR,
            ).unwrap();
    
            let mut frame = frame::Video::empty();
            let mut rgb_frame = frame::Video::empty();
    
            for (stream, packet) in ictx.packets() {
                if stream.index() == stream_index {
                    if let Err(_) = decoder.send_packet(&packet) {
                        continue;
                    }                                                                                                           
    
                    while decoder.receive_frame(&mut frame).is_ok() {
                        scaler.run(&frame, &mut rgb_frame).unwrap();
                        
                        let data = rgb_frame.data(0);
                        let data_len = data.len();
                        // let compressed_data = encode_jpeg(data, rgb_frame.width() as u32, rgb_frame.height() as u32);
                        println!("time: {}, data_len: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"), data_len);
                        app_handle.emit("video_frame", (rgb_frame.width(), rgb_frame.height(), data.to_vec())).ok();
                    }
                }
            }
        });
    }
    
}

#[tauri::command]
pub fn start_video_stream(app: tauri::AppHandle, url: String) {
    let streamer = VideoStreamer::new(app);
    streamer.start_stream(url);
}