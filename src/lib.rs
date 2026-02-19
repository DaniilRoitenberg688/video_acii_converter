mod ascii_converter;
mod cli;
use artem::{ConfigBuilder, config::Config};
use cli::parse;
use color_eyre::{Result, eyre::Context};
use core::num::NonZeroU32;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
    widgets::Paragraph,
};
use std::{sync::mpsc, thread, time::Duration};

use crate::{ascii_converter::*, cli::File};

pub fn run() -> Result<(), String> {
    match parse() {
        Ok(file_name) => {
            if let Err(_) = color_eyre::install() {
                return Err("cannot start application".to_string());
            }; // augment errors / panics with easy to read messages
            let terminal = ratatui::init();
            let app_result = match file_name {
                File::Video(name) => run_tui_video(terminal, name),
                File::Image(name) => run_tui_image(terminal, name),
            };
            ratatui::restore();
            if let Err(_) = app_result {
                return Err("cannot start loop".to_string());
            }
            return Ok(());
        }
        Err(err) => Err(err),
    }
}

fn run_tui_image(mut terminal: DefaultTerminal, file_name: String) -> Result<()> {
    let conf: Config = ConfigBuilder::new()
        .target_size(NonZeroU32::new(200).unwrap())
        .color(false)
        .build();
    let art = convert_image(file_name, &conf);
    loop {
        terminal.draw(|frame: &mut Frame| draw(frame, &art))?;
        if should_quit()? {
            break;
        }
    }
    return Ok(());
}

fn run_tui_video(mut terminal: DefaultTerminal, file_name: String) -> Result<()> {
    //println!("{art}");
    // let images = get_iamges_ascii(vec!["test.png", "test2.png", "test3.png"]);
    //
    let conf: Config = ConfigBuilder::new()
        .target_size(NonZeroU32::new(200).unwrap())
        .color(false)
        .build();

    let (image_t, image_r) = mpsc::channel::<String>();
    let file_name = file_name.to_string(); // Convert &str to owned String
    thread::spawn(move || {
        get_frames_from_video(&file_name, &conf, image_t);
    });
    loop {
        match image_r.recv() {
            Ok(image) => {
                terminal.draw(|frame: &mut Frame| draw(frame, &image))?;
            }
            _ => {
                break;
            }
        }

        if should_quit()? {
            break;
        }
    }
    Ok(())
}

// fn render_colored_artem(art_string: &String) -> Text<'static> {
//     let mut lines = Vec::new();
//     let current_style = Style::default();

//     for line_str in art_string.lines() {
//         let spans: Vec<Span> = line_str
//             .chars()
//             .map(|c| {
//                 let style = Style::default().fg(Color::White);
//                 Span::styled(c.to_string(), style)
//             })
//             .collect();

//         lines.push(Line::from(spans));
//     }

//     Text::from(lines).style(current_style)
// }

fn draw(frame: &mut Frame, text: &String) {
    let greeting = Paragraph::new(text as &str);
    frame.render_widget(greeting, frame.area());
}

fn should_quit() -> Result<bool> {
    if event::poll(Duration::from_millis(1)).context("event poll failed")? {
        if let Event::Key(key) = event::read().context("event read failed")? {
            return Ok(KeyCode::Char('q') == key.code);
        }
    }
    Ok(false)
}
