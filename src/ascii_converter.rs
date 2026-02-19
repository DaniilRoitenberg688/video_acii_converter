use artem::config::Config;
use image::DynamicImage;
use image::{ImageBuffer, ImageReader, Rgb};
use std::{path::Path, sync::mpsc::Sender};
use video_rs::{decode::Decoder, frame::RawFrame};

#[allow(dead_code)]
pub fn convert_image(name: String, conf: &Config) -> String {
    let i = ImageReader::open(name).unwrap().decode().unwrap();
    let art = artem::convert(i, &conf);
    return art;
}

#[allow(dead_code)]
pub fn get_images_ascii(images: Vec<String>, conf: &Config) -> Vec<String> {
    let mut res: Vec<String> = vec![];
    for i in images {
        res.push(convert_image(i, conf));
    }
    return res;
}

pub fn get_frames_from_video(name: &str, conf: &Config, image_t: Sender<String>) {
    let mut frame_counter = 0;
    let mut decoder = Decoder::new(Path::new(name)).unwrap();
    for i in decoder.decode_raw_iter() {
        if frame_counter % 2 == 0 {
            match i {
                Ok(frame) => {
                    let art = convert_frame_to_ascii(frame, conf);
                    image_t.send(art).unwrap();
                }
                _ => break,
            }
        }
        frame_counter += 1;
    }
}

pub fn convert_frame_to_ascii(frame: RawFrame, conf: &Config) -> String {
    let h = frame.height() as u32;
    let w = frame.width() as u32;
    let mut img_buf = ImageBuffer::new(w as u32, h as u32);
    let linesize = frame.stride(0);
    let data = frame.data(0);
    for y in 0..h {
        let src_row_offset = (y * linesize as u32) as usize;
        for x in 0..w {
            let src_idx = src_row_offset + (x as usize * 3); // 3 bytes per pixel for RGB24

            // Check bounds to prevent panic
            if src_idx + 2 < data.len() {
                let r = data[src_idx];
                let g = data[src_idx + 1];
                let b = data[src_idx + 2];
                img_buf.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
    }
    let dyn_image = DynamicImage::ImageRgb8(img_buf);
    let res = artem::convert(dyn_image, conf);
    res
}
