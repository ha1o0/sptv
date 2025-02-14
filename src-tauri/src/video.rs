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
            let mut scaler = scaling::Context::get(
                decoder.format(),
                decoder_width,
                decoder_height,
                Pixel::RGB24, // 使用 RGB24 格式
                1920,
                1920 * decoder_height / decoder_width, // 降低分辨率
                scaling::Flags::BILINEAR,
            )
            .unwrap();

            let mut frame = frame::Video::empty();
            let mut rgb_frame = frame::Video::empty();

            for (stream, packet) in ictx.packets() {
                if stream.index() == stream_index {
                    if let Err(e) = decoder.send_packet(&packet) {
                        eprintln!("Failed to send packet: {}", e);
                        continue;
                    }

                    while decoder.receive_frame(&mut frame).is_ok() {
                        scaler.run(&frame, &mut rgb_frame).unwrap();

                        // 将 RGB24 帧数据转换为 image::RgbImage
                        let width = rgb_frame.width() as u32;
                        let height = rgb_frame.height() as u32;
                        let data = rgb_frame.data(0);

                        // let image_buffer: RgbImage = ImageBuffer::from_raw(width, height, data.to_vec())
                        //     .expect("Failed to create image buffer");

                        // 编码为 JPEG 格式
                        // let mut jpeg_data = Vec::new();
                        // image::codecs::jpeg::JpegEncoder::new(&mut jpeg_data)
                        //     .encode_image(&image_buffer)
                        //     .expect("Failed to encode JPEG");

                        // if jpeg_data.len() > 2 && jpeg_data[0] == 0xFF && jpeg_data[1] == 0xD8 &&
                        //     jpeg_data[jpeg_data.len() - 2] == 0xFF && jpeg_data[jpeg_data.len() - 1] == 0xD9 {
                        //      println!("JPEG 数据完整");
                        //     //  VideoStreamer::save_jpeg_test(&jpeg_data);
                        //  } else {
                        //      println!("JPEG 数据不完整");
                        //  }
                        // 或者编码为 PNG 格式
                        // let mut png_data = Vec::new();
                        // image::codecs::png::PngEncoder::new(&mut png_data)
                        //     .encode_image(&image_buffer)
                        //     .expect("Failed to encode PNG");

                        // println!("image size: {}", jpeg_data.len());

                        // 创建共享内存
                        let shmem = ShmemConf::new().size(data.len()).create().unwrap();
                        let frame_data = std::sync::Arc::new(Mutex::new(shmem));

                        // 写入帧数据
                        let mut frame = frame_data.lock().unwrap();
                        unsafe { frame.as_slice_mut().copy_from_slice(&data) };

                        // 从 frame_data 中获取 shmem 的引用
                        let os_id = {
                            frame.get_os_id() // 获取 os_id
                        };

                        println!(
                            "time: {}, ori_data_len: {}, os_id: {}",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                            data.len(),
                            os_id
                        );

                        app_handle
                            .emit("video_frame", (width, height, data))
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
