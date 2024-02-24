use clap::Parser;
use rodio::{
    Sink,
    OutputStream,
    Source,
    Decoder
};
use std::{
    fs::metadata,
    thread,
    time::Duration,
};

/// A program to play a song in the background
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Filepath of track to play
    #[arg(short, long)]
    title: String,

    /// Control loop setting
    #[arg(short, long, default_value_t = 0, requires("title"))]
    doloop: u8,
}

fn file_exists(file_path: &str) -> bool { metadata(file_path).is_ok() }
fn play_audio(file_path: &str, loop_enabled: bool) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let file = std::fs::File::open(file_path).unwrap();
    let source = Decoder::new(std::io::BufReader::new(file)).unwrap();

    if loop_enabled {
        // if user wants a loop create a looped version of the source
        let infinite_source = source.repeat_infinite();
        thread::sleep(Duration::from_secs(1));
        println!("playme: looping");
        loop {
            sink.append(infinite_source.clone());
            if sink.empty() {
                thread::sleep(Duration::from_secs(5));
            }
        }
    } else {
        // if not looping, wait for the audio to finish playing
        sink.append(source);
        thread::sleep(Duration::from_secs(10));
        println!("playme: completed...");
    }
    
    sink.stop();
}

fn main() {
    let args = Args::parse();

    let doiplay: bool = match args.doloop {
        1 => true,
        0 => false,
        _ => {
            println!("Unexpected value");
            false
        }
    };

    // check if the file exists before attempting to play it
    if file_exists(args.title.as_str()) {
        play_audio(&args.title, doiplay);
    } else {
        eprintln!("File '{}' does not exist.", args.title);
    }
}
