use artem::config::Config;
use image::ImageReader;
use ndarray::{Array2, s};
use std::{path::Path, sync::mpsc::Sender};
use video_rs::Frame;
use video_rs::decode::Decoder;

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

pub fn get_frames_from_video(name: &str, image_t: Sender<String>) {
    let mut decoder = Decoder::new(Path::new(name)).unwrap();
    // let mut frame_count = 0;
    while let Ok((_, frame)) = decoder.decode() {
        // let a = frame.timestamp().unwrap();
        let res = convert_frame_to_ascii(frame);
        image_t.send(res).unwrap();
    }
}

pub fn convert_frame_to_ascii(frame: Frame) -> String {
    let mut res = String::new();
    let k = 0.0045;
    let s = frame.shape();
    let height = s[0];
    let width = s[1];
    let depth = s[2];
    let bucket_width = (width as f32 * k) as usize;
    let bucket_height = ((height as f32 * k) * (height as f32 / width as f32)) as usize;
    for y in 0..(height / bucket_height - 1) {
        for x in 0..(width / bucket_width - 1) {
            let bucket = frame.slice(s![
                (y * bucket_height)..((y + 1) * bucket_height),
                (x * bucket_width)..((x + 1) * bucket_width),
                ..
            ]);
            let con = bucket.to_shape((bucket_height * bucket_width, 3)).unwrap();
            let con = con.mapv(|x| x as usize);
            let r = con.column(0).sum() / (bucket_height * bucket_width);
            let g = con.column(1).sum() / (bucket_height * bucket_width);
            let b = con.column(2).sum() / (bucket_height * bucket_width);
            let grey = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
            let sym = convert_grey_to_char(grey);
            res.push(sym);
        }
        res.push('\n');
    }
    res
}

fn convert_grey_to_char(grey: f32) -> char {
    let possible_syms: Vec<char> = String::from("@%#*+=-:. ").chars().collect();
    let for_one = 255.0 / (possible_syms.len() - 1) as f32;
    let ind = (grey / for_one).ceil() as usize;
    possible_syms[possible_syms.len() - 1 - ind]
}
