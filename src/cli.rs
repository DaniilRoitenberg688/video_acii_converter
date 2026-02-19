use std::path::Path;

use clap::Parser;

pub enum File {
    Image(String),
    Video(String),
}

#[derive(Debug, Parser)]
struct Data {
    filename: String,
}

pub fn parse() -> Result<File, String> {
    let data = Data::parse();
    let path_path = Path::new(&data.filename);
    if !path_path.exists() {
        return Err("cannot find file".to_string());
    }
    if let Some(exte) = path_path.extension() {
        match exte.to_str() {
            Some("mp4") => return Ok(File::Video(data.filename)),
            Some("png") | Some("jpg") | Some("jpeg") => return Ok(File::Image(data.filename)),
            _ => return Err("unknown filetype".to_string()),
        }
    }
    return Err("something went wrong".to_string());
}
