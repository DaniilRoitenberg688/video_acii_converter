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
use std::{path::Path, sync::mpsc, thread, time::Duration};
use video_rs::Decoder;

use crate::{ascii_converter::*, cli::File};

pub fn run(size: u32) -> Result<(), String> {
    match parse() {
        Ok(file_name) => {
            if color_eyre::install().is_err() {
                return Err("cannot start application".to_string());
            }; // augment errors / panics with easy to read messages
            let terminal = ratatui::init();
            let app_state = AppState::new();
            let app_result = match file_name {
                File::Video(name) => run_tui_video_colored(app_state, terminal, name),
                File::Image(name) => run_tui_image(app_state, terminal, name, size),
            };
            ratatui::restore();
            if let Err(e) = app_result {
                println!("{e}");
                return Err(format!("cannot start loop: due to this error {}", e).to_string());
            }
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn run_tui_image(
    mut app_state: AppState,
    mut terminal: DefaultTerminal,
    file_name: String,
    size: u32,
) -> Result<()> {
    let conf: Config = ConfigBuilder::new()
        .target_size(NonZeroU32::new(size).unwrap())
        .color(false)
        .build();
    let art = convert_image(file_name, &conf);
    loop {
        terminal.draw(|frame: &mut Frame| draw(frame, &art))?;
        let _ = app_state.update();
        if app_state.is_quit {
            break;
        }
    }
    Ok(())
}

fn run_tui_video_colored(
    mut app_state: AppState,
    mut terminal: DefaultTerminal,
    file_name: String,
) -> Result<()> {
    let (image_t, image_r) = mpsc::channel::<Paragraph<'static>>();
    let mut decoder = match Decoder::new(Path::new(&file_name)) {
        Ok(a) => a,
        Err(e) => {
            return Result::Err(e.into());
        }
    };
    // let loaded_frames: Arc<Mutex<HashMap<usize, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let t = thread::spawn(move || {
        get_frames_from_video_colored(&mut decoder, image_t);
    });
    loop {
        if let Ok(art) = image_r.recv()
            && !app_state.is_paused
        {
            terminal.draw(|frame: &mut Frame| draw_colored(frame, art))?;
        }
        if app_state.is_quit {
            break;
        }
        _ = app_state.update();
    }
    drop(t);
    Ok(())
}

fn run_tui_video(
    mut app_state: AppState,
    mut terminal: DefaultTerminal,
    file_name: String,
) -> Result<()> {
    let (image_t, image_r) = mpsc::channel::<String>();
    let mut decoder = match Decoder::new(Path::new(&file_name)) {
        Ok(a) => a,
        Err(e) => {
            return Result::Err(e.into());
        }
    };
    // let loaded_frames: Arc<Mutex<HashMap<usize, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let t = thread::spawn(move || {
        get_frames_from_video(&mut decoder, image_t);
    });
    loop {
        if let Ok(art) = image_r.recv()
            && !app_state.is_paused
        {
            terminal.draw(|frame: &mut Frame| draw(frame, &art))?;
        }
        _ = app_state.update();
        if app_state.is_quit {
            break;
        }
    }
    drop(t);
    Ok(())
}

struct AppState {
    is_paused: bool,
    is_quit: bool,
    speed: bool,
}

impl AppState {
    fn new() -> AppState {
        AppState {
            is_paused: false,
            is_quit: false,
            speed: false,
        }
    }
    fn update(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(33)).context("event poll failed")? {
            if let Event::Key(key) = event::read().context("event read failed")? {
                match key.code {
                    KeyCode::Char('q') => {
                        self.is_quit = true;
                    }
                    KeyCode::Char(' ') => {
                        self.is_paused = !self.is_paused;
                    }
                    KeyCode::Right => {
                        if key.is_press() {
                            self.speed = true;
                        } else if key.is_release() {
                            self.speed = false;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

fn draw(frame: &mut Frame, text: &str) {
    let greeting = Paragraph::new(text as &str);
    frame.render_widget(greeting, frame.area());
}

fn draw_colored(frame: &mut Frame, text: Paragraph<'static>) {
    frame.render_widget(text, frame.area());
}
