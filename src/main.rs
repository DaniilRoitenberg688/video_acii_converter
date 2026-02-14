use artem::config::{Config, ConfigBuilder};
use ndarray::Array3;
use core::num::NonZeroU32;
use image::{ImageBuffer, ImageReader, Rgb};
use std::{path::Path, time::Duration};
use video_rs::{decode::Decoder, frame::{self, RawFrame}};

use color_eyre::{eyre::Context, Result};
use image::DynamicImage;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    DefaultTerminal, Frame,
};

fn convert_image(name: &str, conf: &Config) -> String {
    let i = ImageReader::open(name).unwrap().decode().unwrap();
    let art = artem::convert(i, &conf);
    return art;
}

fn get_images_ascii(images: Vec<&str>, conf: &Config) -> Vec<String> {
    let mut res: Vec<String> = vec![];
    for i in images {
        res.push(convert_image(i, conf));
    }
    return res;
}

fn get_frames_from_video(name: &str, conf: &Config) -> Vec<String> {
    let mut res: Vec<String> = vec![];
    let mut decoder = Decoder::new(Path::new(name)).unwrap();
    for i in decoder.decode_raw_iter() {
        match i {
            Ok(frame) => {
                let art = convert_frame_to_ascii(frame, conf);
                res.push(art);
            },
            _ => break
        }
    }
    return res;
}

fn convert_frame_to_ascii(frame: RawFrame, conf: &Config) -> String {
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

fn main() -> Result<()> {
    color_eyre::install()?; // augment errors / panics with easy to read messages
    let terminal = ratatui::init();
    let app_result = run(terminal).context("app loop failed");
    ratatui::restore();
    app_result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    //println!("{art}");
    // let images = get_iamges_ascii(vec!["test.png", "test2.png", "test3.png"]);
    //
    let conf: Box<Config> = Box::new(
        ConfigBuilder::new()
            .target_size(NonZeroU32::new(250).unwrap())
            .color(false)
            .build(),
    );
    let images = get_frames_from_video("hakari.mp4", &conf);
    let mut index = 0;
    loop {
        terminal.draw(|frame: &mut Frame| draw(frame, &images[index % images.len()]))?;
        if should_quit()? {
            break;
        }
        if index >= images.len() {
            break;
        }
        index += 1;
    }
    Ok(())
}

fn render_colored_artem(art_string: &String) -> Text<'static> {
    let mut lines = Vec::new();
    let current_style = Style::default();

    for line_str in art_string.lines() {
        let spans: Vec<Span> = line_str
            .chars()
            .map(|c| {
                let style = Style::default().fg(Color::White);
                Span::styled(c.to_string(), style)
            })
            .collect();

        lines.push(Line::from(spans));
    }

    Text::from(lines).style(current_style)
}

fn draw(frame: &mut Frame, text: &String) {
    let data = render_colored_artem(text);
    let greeting = Paragraph::new(data);
    frame.render_widget(greeting, frame.area());
}

fn should_quit() -> Result<bool> {
    if event::poll(Duration::from_millis(30)).context("event poll failed")? {
        if let Event::Key(key) = event::read().context("event read failed")? {
            return Ok(KeyCode::Char('q') == key.code);
        }
    }
    Ok(false)
}
