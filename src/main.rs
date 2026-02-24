use video_processor::run;
const SIZE: u32 = 200;

fn main() {
    if let Err(e) = run(SIZE) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
