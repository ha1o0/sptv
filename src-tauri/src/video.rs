use ffmpeg_next::{codec, format, frame, software::scaling, util::format::Pixel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter}; // 导入 Emitter

struct VideoStreamer {
    app_handle: AppHandle,
}

impl VideoStreamer {
    pub fn new(app_handle: AppHandle) -> Self {
        VideoStreamer { app_handle }
    }

    pub fn start_stream(&self, url: String) {
        // 将 app_handle 包装成 Arc<Mutex> 来在多个线程之间共享
        let app_handle = Arc::new(Mutex::new(self.app_handle.clone()));

        thread::spawn(move || {
            ffmpeg_next::init().unwrap();

            let mut ictx = format::input(&url).expect("无法打开 M3U8 流");

            let stream = ictx
                .streams()
                .best(ffmpeg_next::media::Type::Video)
                .expect("没有找到视频流");
            let stream_index = stream.index();

            let mut decoder = codec::Context::from_parameters(stream.parameters())
                .unwrap()
                .decoder()
                .video()
                .unwrap();

            let mut scaler = scaling::Context::get(
                decoder.format(),
                decoder.width(),
                decoder.height(),
                Pixel::RGB24,
                decoder.width(),
                decoder.height(),
                scaling::Flags::BILINEAR,
            )
            .unwrap();

            let mut frame = frame::Video::empty();
            let mut rgb_frame = frame::Video::empty();

            for (stream, packet) in ictx.packets() {
                if stream.index() == stream_index {
                    decoder.send_packet(&packet).unwrap();
                    while decoder.receive_frame(&mut frame).is_ok() {
                        scaler.run(&frame, &mut rgb_frame).unwrap();

                        let data = rgb_frame.data(0);
                        let width = rgb_frame.width();
                        let height = rgb_frame.height();
                        // println!("{}, {}, {:?}", width, height, data);

                        // 使用 Arc<Mutex> 来访问 app_handle
                        let app_handle = app_handle.lock().unwrap();
                        app_handle
                            .emit("video_frame", (width, height, data.to_vec()))
                            .expect("事件发送失败");
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
    }
}

// Tauri 命令：启动 M3U8 直播流
#[tauri::command]
pub fn start_video_stream(app: tauri::AppHandle, url: String) {
    let streamer = VideoStreamer::new(app);
    streamer.start_stream(url);
}
