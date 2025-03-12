pub mod ring_buffer;
pub mod ts_cache;
pub mod ts_cache_manager;

use ffmpeg::codec::context::Context as CodecContext;
use ffmpeg_next::decoder::Video;
use ffmpeg_next::{self as ffmpeg, Rational};
use ffmpeg_next::{
    codec::{self},
    format, frame,
    software::scaling,
    util::format::Pixel,
    Codec,
};
use lazy_static::lazy_static;
use ring_buffer::{RingBufferManager, VideoFrame};
// use image::{ImageBuffer, RgbImage};
// use std::fs::File;
// use std::io::Write;
use shared_memory::*;
use std::fmt::Debug;
use std::io::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};
use tokio::time::sleep;
use ts_cache::TsCache;

use crate::AppState;

lazy_static! {
    static ref GLOBAL_RING_BUFFER: Arc<Mutex<RingBufferManager>> =
        Arc::new(Mutex::new(RingBufferManager::new(20)));
}

struct VideoStreamer {
    app_handle: AppHandle,
}

impl VideoStreamer {
    pub fn new(app_handle: AppHandle) -> Self {
        VideoStreamer { app_handle }
    }

    /// Get the `time_base` field of an encoder. (Not natively supported in the public API.)
    pub fn _get_encoder_time_base(encoder: &Video) -> Rational {
        unsafe { (*encoder.0.as_ptr()).time_base.into() }
    }
    /// Initialize a new codec context using a specific codec.
    pub fn codec_context_as(codec: &Option<Codec>) -> Option<CodecContext> {
        match codec {
            None => Some(CodecContext::new()),
            Some(codec) => unsafe {
                let context_ptr = ffmpeg::ffi::avcodec_alloc_context3(codec.as_ptr());
                println!("context_ptr: {:?}", context_ptr);
                if !context_ptr.is_null() {
                    println!("context_ptr:not null");
                    Some(CodecContext::wrap(context_ptr, None))
                } else {
                    None
                }
            },
        }
    }

    fn select_best_decoder(codec_id: ffmpeg_next::ffi::AVCodecID) -> Option<Codec> {
        let result = match codec_id {
            ffmpeg_next::ffi::AVCodecID::AV_CODEC_ID_H264 => {
                codec::decoder::find_by_name("h264_qsv")
                    .or_else(|| codec::decoder::find_by_name("h264_amf"))
                    .or_else(|| codec::decoder::find_by_name("h264_cuvid"))
                    .or_else(|| None)
            }
            ffmpeg_next::ffi::AVCodecID::AV_CODEC_ID_HEVC => {
                codec::decoder::find_by_name("hevc_qsv")
                    .or_else(|| codec::decoder::find_by_name("hevc_vaapi"))
                    .or_else(|| codec::decoder::find_by_name("hevc_amf"))
                    .or_else(|| None)
            }
            ffmpeg_next::ffi::AVCodecID::AV_CODEC_ID_VP9 => {
                codec::decoder::find_by_name("vp9_cuvid")
                    .or_else(|| codec::decoder::find_by_name("vp9_vaapi"))
                    .or_else(|| None)
            }
            ffmpeg_next::ffi::AVCodecID::AV_CODEC_ID_AV1 => {
                codec::decoder::find_by_name("av1_cuvid")
                    .or_else(|| codec::decoder::find_by_name("av1_vaapi"))
                    .or_else(|| None)
            }
            _ => None,
        };

        result
    }

    pub async fn decode_video_file(file_url: String, url_id: String) {
        let handle = thread::spawn(move || {
            println!("解码线程已启动");
            let mut ictx = format::input(&url_id).expect("无法打开视频流");
            let stream = ictx
                .streams()
                .best(ffmpeg_next::media::Type::Video)
                .expect("没有找到视频流");
            let stream_index = stream.index();
            let fps = stream.avg_frame_rate();
            let frame_rate = fps.numerator() / fps.denominator();
            println!("帧率: {}", frame_rate);

            let decoder_width = 1920;
            let decoder_height = 1080;

            let codec_id = stream.parameters().id();
            println!("codec_id: {:?}", codec_id);
            // 尝试选择最佳硬件解码器
            let best_coder = Self::select_best_decoder(codec_id.into());
            let mut is_hardware_decoder = false;
            let mut decoder = if let Some(c) = &best_coder {
                println!("使用的硬件解码器: {}", c.name());
                is_hardware_decoder = true;
                Self::codec_context_as(&best_coder)
                    .unwrap()
                    .decoder()
                    .video()
                    .unwrap()
            } else {
                // 回退到软件解码器
                println!("未找到合适的硬件解码器，回退到软件解码器");
                codec::Context::from_parameters(stream.parameters())
                    .unwrap()
                    .decoder()
                    .video()
                    .unwrap()
            };

            let decoder_format = if is_hardware_decoder {
                Pixel::NV12
            } else {
                decoder.format()
            };
            println!("解码器格式: {:?}", decoder_format);
            if decoder_width == 0 || decoder_height == 0 {
                eprintln!(
                    "Invalid decoder dimensions: {}x{}",
                    decoder_width, decoder_height
                );
            }

            let dst_width = 1920;
            let dst_height =
                ((dst_width as f32 * decoder_height as f32 / decoder_width as f32) as u32 + 1) & !1;

            let mut scaler = scaling::Context::get(
                decoder_format,
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
                        let y_data = yuv_frame.data(0).to_vec();
                        let u_data = yuv_frame.data(1).to_vec();
                        let v_data = yuv_frame.data(2).to_vec();
                        let video_frame = VideoFrame {
                            y_plane: y_data,
                            u_plane: u_data,
                            v_plane: v_data,
                            width: dst_width,
                            height: dst_height,
                        };
                        // 循环检查直到buffer有空间
                        loop {
                            if let Ok(mut manager) = GLOBAL_RING_BUFFER.lock() {
                                let buffer_size = manager.get_buffer_size(&url_id);
                                if buffer_size < 20 {
                                    // RingBufferManager初始化时设置的容量
                                    // println!("video_frame - y_plane len: {}, u_plane len: {}, v_plane len: {}", video_frame.y_plane.len(), video_frame.u_plane.len(), video_frame.v_plane.len());
                                    manager.push(&url_id, video_frame);
                                    // println!("buffer_size: {}, {}", buffer_size, &url_id);
                                    break;
                                }
                                // 如果buffer满了，释放锁并等待一段时间再重试
                                // drop(manager);
                                // println!("Buffer已满，等待空间...");
                                // std::thread::sleep(std::time::Duration::from_millis(10));
                            }
                        }
                    }
                    // println!(
                    //     "time: {}, Frame pushed to ring buffer for url: {}",
                    //     chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    //     &file_url
                    // );
                }
            }
        });
        handle.join().unwrap(); // 等待子线程完成
        println!("解码完成");
        // 解码完成后，将ts_cache中当前直播流的第一个ts文件从ts_cache中移除
        // let _ = ts_cache.remove_first_ts().await;
    }

    pub async fn start_stream(&self, url: String) {
        println!("start_stream url: {:?}", url);
        ffmpeg_next::init().unwrap();
        let app_handle = self.app_handle.clone();
        app_handle
            .emit("video_frame", (0, 0, 0, 0, 0))
            .expect("Failed to emit video frame");
        Self::decode_video_file("".to_owned(), url).await;
        // let mut current_decode_url = url.clone();
        // loop {
        //     let (ts_cache_url, _sequence, duration) = ts_cache.get_next_ts().await;
        //     println!("ts_cache_url: {:?}", ts_cache_url);
        //     if ts_cache_url != "" {
        //         let ts_cache_url_clone = ts_cache_url.clone();
        //         if ts_cache_url == current_decode_url {
        //             sleep(std::time::Duration::from_secs(duration - 1)).await;
        //             continue;
        //         }
        //         Self::decode_video_file(ts_cache_url, url.clone(), ts_cache.clone()).await;
        //         current_decode_url = ts_cache_url_clone;
        //     } else {
        //         // todo 这里需要处理一下间隔时间
        //         sleep(std::time::Duration::from_secs(1)).await;
        //     }
        // }
        // ffmpeg_next::init().unwrap();
        // thread::spawn(move || {
        //     let mut ictx = format::input(&url).expect("无法打开 M3U8 流");
        //     let stream = ictx
        //         .streams()
        //         .best(ffmpeg_next::media::Type::Video)
        //         .expect("没有找到视频流");
        //     let stream_index = stream.index();
        //     let fps = stream.avg_frame_rate();
        //     let frame_rate = fps.numerator() / fps.denominator();
        //     println!("帧率: {}", frame_rate);

        //     let decoder_width = 1920;
        //     let decoder_height = 1080;

        //     let codec_id = stream.parameters().id();
        //     println!("codec_id: {:?}", codec_id);
        //     // 尝试选择最佳硬件解码器
        //     let best_coder = Self::select_best_decoder(codec_id.into());
        //     let mut is_hardware_decoder = false;
        //     let mut decoder = if let Some(c) = &best_coder {
        //         println!("使用的硬件解码器: {}", c.name());
        //         is_hardware_decoder = true;
        //         Self::codec_context_as(&best_coder)
        //             .unwrap()
        //             .decoder()
        //             .video()
        //             .unwrap()
        //     } else {
        //         // 回退到软件解码器
        //         println!("未找到合适的硬件解码器，回退到软件解码器");
        //         codec::Context::from_parameters(stream.parameters())
        //             .unwrap()
        //             .decoder()
        //             .video()
        //             .unwrap()
        //     };

        //     let decoder_format = if is_hardware_decoder {
        //         Pixel::NV12
        //     } else {
        //         decoder.format()
        //     };
        //     println!("解码器格式: {:?}", decoder_format);
        //     if decoder_width == 0 || decoder_height == 0 {
        //         eprintln!(
        //             "Invalid decoder dimensions: {}x{}",
        //             decoder_width, decoder_height
        //         );
        //     }

        //     let dst_width = 640;
        //     let dst_height =
        //         ((dst_width as f32 * decoder_height as f32 / decoder_width as f32) as u32 + 1) & !1;

        //     let mut scaler = scaling::Context::get(
        //         decoder_format,
        //         decoder_width,
        //         decoder_height,
        //         Pixel::YUV420P,
        //         dst_width,
        //         dst_height,
        //         scaling::Flags::BILINEAR,
        //     )
        //     .unwrap();

        //     let mut frame = frame::Video::empty();
        //     let mut yuv_frame = frame::Video::new(Pixel::YUV420P, dst_width, dst_height);

        //     for (stream, packet) in ictx.packets() {
        //         if stream.index() == stream_index {
        //             if let Err(e) = decoder.send_packet(&packet) {
        //                 eprintln!("Failed to send packet: {}", e);
        //                 continue;
        //             }

        //             while decoder.receive_frame(&mut frame).is_ok() {
        //                 scaler.run(&frame, &mut yuv_frame).unwrap();

        //                 let y_data = yuv_frame.data(0).to_vec();
        //                 let u_data = yuv_frame.data(1).to_vec();
        //                 let v_data = yuv_frame.data(2).to_vec();

        //                 let y_size = (dst_width * dst_height) as usize;
        //                 let uv_size = y_size / 4;

        //                 assert_eq!(y_data.len(), y_size);
        //                 assert_eq!(u_data.len(), uv_size);
        //                 assert_eq!(v_data.len(), uv_size);

        //                 println!(
        //                     "time: {}, y_data_len: {}, u_data_len: {}, v_data_len: {}, width: {}, height: {}",
        //                     chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        //                     y_data.len(),
        //                     u_data.len(),
        //                     v_data.len(),
        //                     dst_width,
        //                     dst_height,
        //                 );

        //                 app_handle
        //                     .emit(
        //                         "video_frame",
        //                         (y_data, u_data, v_data, dst_width, dst_height),
        //                     )
        //                     .expect("Failed to emit video frame");
        //             }
        //         }
        //     }
        // });
    }
}

#[tauri::command]
pub async fn start_video_stream(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    url: String,
) -> Result<(), String> {
    // let cache_manager = state.ts_cache_manager.clone();
    // let url_ts_cache = cache_manager.get_or_create_cache(url.clone()).await;
    let streamer = VideoStreamer::new(app);
    streamer.start_stream(url).await;
    Ok(())
}

#[tauri::command]
pub fn get_video_frame(frame_os_id: &str) -> Vec<u8> {
    return get_video_frame_test(frame_os_id);
}

#[tauri::command]
pub fn test_frame_data() -> Vec<u8> {
    generate_vec(2_073_600)
}

#[tauri::command]
pub fn test_frame_data2(request: tauri::ipc::Request<'_>) -> tauri::ipc::Response {
    if let tauri::ipc::InvokeBody::Raw(_data) = request.body() {
        let url = request.headers().get("url").unwrap().to_str().unwrap();
        // println!("url from request: {:?}", url);
        // 获取全局 ring buffer 的锁
        if let Ok(mut manager) = GLOBAL_RING_BUFFER.lock() {
            // 从 ring buffer 中获取第一帧数据
            if let Some(frame) = manager.pop_n(url, 1).first() {
                // println!("frame y size: {:?}", frame.y_plane.len());
                // println!("frame u size: {:?}", frame.u_plane.len());

                // 将 YUV 数据平面合并成一个向量
                let mut frame_data = Vec::new();
                frame_data.extend_from_slice(&frame.y_plane);
                frame_data.extend_from_slice(&frame.u_plane);
                frame_data.extend_from_slice(&frame.v_plane);

                return tauri::ipc::Response::new(frame_data);
            }
            // println!("没有数据0");
            return tauri::ipc::Response::new(vec![]);
        }
        // println!("没有数据1");
        tauri::ipc::Response::new(vec![])
    } else {
        // println!("没有数据2");
        tauri::ipc::Response::new(vec![])
    }
}

pub fn generate_vec(size: i32) -> Vec<u8> {
    println!(
        "开始生成时间: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f")
    );
    let mut rng = rand::thread_rng();
    let arr: Vec<u8> = (0..size)
        .map(|_| rand::Rng::gen_range(&mut rng, 0..10))
        .collect();
    println!(
        "结束生成时间: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f")
    );
    arr
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
