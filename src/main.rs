use video_processor::run;

fn main() {
    if let Err(e) = run("static/rover.mp4") {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
