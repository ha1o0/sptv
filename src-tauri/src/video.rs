use ffmpeg_next::{codec, format, frame, software::scaling, util::format::Pixel};
// use image::{ImageBuffer, RgbImage};
// use std::fs::File;
// use std::io::Write;
use shared_memory::*;
use std::sync::Mutex;
use std::thread;
use tauri::{AppHandle, Emitter};

struct VideoStreamer {
    app_handle: AppHandle,
}

impl VideoStreamer {
    pub fn new(app_handle: AppHandle) -> Self {
        VideoStreamer { app_handle }
    }

    // pub fn save_jpeg_test(data: &[u8]) {
    //     let mut file = File::create("test.jpg").expect("Failed to create file");
    //     file.write_all(data).expect("Failed to write data");
    // }

    pub fn start_stream(&self, url: String) {
        let app_handle = self.app_handle.clone();
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

            let decoder_width = decoder.width();
            let decoder_height = decoder.height();

            // 计算目标尺寸，保持宽高比
            let dst_width = 1920; // 或其他期望的宽度
            let dst_height =
                (dst_width as f32 * decoder_height as f32 / decoder_width as f32) as u32;
            // 确保高度是偶数（YUV420P 要求）
            let dst_height = (dst_height + 1) & !1;

            let mut scaler = scaling::Context::get(
                decoder.format(),
                decoder_width,
                decoder_height,
                Pixel::YUV420P,
                dst_width,
                dst_height,
                scaling::Flags::BILINEAR,
            )
            .unwrap();

            let mut frame = frame::Video::empty();
            let mut yuv_frame = frame::Video::new(Pixel::YUV420P, dst_width, dst_height);

            for (stream, packet) in ictx.packets() {
                if stream.index() == stream_index {
                    if let Err(e) = decoder.send_packet(&packet) {
                        eprintln!("Failed to send packet: {}", e);
                        continue;
                    }

                    while decoder.receive_frame(&mut frame).is_ok() {
                        scaler.run(&frame, &mut yuv_frame).unwrap();

                        // 获取 YUV 数据
                        let y_data = yuv_frame.data(0).to_vec(); // Y 分量
                        let u_data = yuv_frame.data(1).to_vec(); // U 分量
                        let v_data = yuv_frame.data(2).to_vec(); // V 分量

                        // 计算正确的 plane sizes
                        let y_size = (dst_width * dst_height) as usize;
                        let uv_size = y_size / 4; // YUV420P 格式中 U 和 V 平面大小是 Y 平面的 1/4

                        // 确保数据长度正确
                        assert_eq!(y_data.len(), y_size);
                        assert_eq!(u_data.len(), uv_size);
                        assert_eq!(v_data.len(), uv_size);

                        println!(
                            "time: {}, y_data_len: {}, u_data_len: {}, v_data_len: {}, width: {}, height: {}",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                            y_data.len(),
                            u_data.len(),
                            v_data.len(),
                            dst_width,
                            dst_height,
                        );

                        app_handle
                            .emit(
                                "video_frame",
                                (y_data, u_data, v_data, dst_width, dst_height),
                            )
                            .expect("Failed to emit video frame");
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

#[tauri::command]
pub fn get_video_frame(frame_os_id: &str) -> Vec<u8> {
    return get_video_frame_test(frame_os_id);
}

pub fn get_video_frame_test(os_id: &str) -> Vec<u8> {
    // 通过 os_id 打开共享内存
    let shmem = ShmemConf::new()
        .os_id(os_id)
        .open()
        .expect("Failed to open shared memory");
    // 从共享内存中读取数据
    let slice = unsafe { shmem.as_slice() };
    slice.to_vec()
}
