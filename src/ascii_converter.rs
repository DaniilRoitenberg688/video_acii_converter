use artem::config::Config;
use image::ImageReader;
use ndarray::s;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use std::sync::mpsc::Sender;
use video_rs::Frame;
use video_rs::decode::Decoder;

#[allow(dead_code)]
pub fn convert_image(name: String, conf: &Config) -> String {
    let i = ImageReader::open(name).unwrap().decode().unwrap();
    artem::convert(i, conf)
}

#[allow(dead_code)]
pub fn get_images_ascii(images: Vec<String>, conf: &Config) -> Vec<String> {
    let mut res: Vec<String> = vec![];
    for i in images {
        res.push(convert_image(i, conf));
    }
    res
}

pub fn get_frames_from_video(decoder: &mut Decoder, image_t: Sender<String>) {
    // let mut frame_count = 0;
    println!("ok");
    while let Ok((_, frame)) = decoder.decode() {
        // let a = frame.timestamp().unwrap();
        let res = convert_frame_to_ascii(frame);
        image_t.send(res).unwrap();
    }
}

pub fn get_frames_from_video_colored(decoder: &mut Decoder, image_t: Sender<Paragraph<'static>>) {
    // let mut frame_count = 0;
    while let Ok((_, frame)) = decoder.decode() {
        // let a = frame.timestamp().unwrap();
        let res = convert_frame_to_ascii_colored(frame);
        image_t.send(res).unwrap();
    }
}

pub fn convert_frame_to_ascii_colored(frame: Frame) -> Paragraph<'static> {
    let k = 0.017;
    let mut res = Vec::new();
    let s = frame.shape();
    let height = s[0];
    let width = s[1];
    // let depth = s[2];
    let bucket_width = (width as f32 * k * (height as f32 / width as f32)) as usize;
    let bucket_height = (height as f32 * k * (width as f32 / height as f32)) as usize;
    for y in 0..(height / bucket_height - 1) {
        let mut l_s = Vec::new();
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
            let color = Color::Rgb(r as u8, g as u8, b as u8);
            let spn = Span::styled(sym.to_string(), Style::default().fg(color));
            l_s.push(spn);
        }
        let line = Line::from(l_s);
        res.push(line);
    }
    Paragraph::new(Text::from(res))
}

pub fn convert_frame_to_ascii(frame: Frame) -> String {
    let mut res = String::new();
    let k = 0.006;
    let s = frame.shape();
    let height = s[0];
    let width = s[1];
    // let depth = s[2];
    let bucket_width = (width as f32 * k * (height as f32 / width as f32)) as usize;
    let bucket_height = (height as f32 * k * (width as f32 / height as f32)) as usize;
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
