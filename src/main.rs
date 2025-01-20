use clap::Parser;
use daemonize::Daemonize;
use rodio::{
    Sink,
    OutputStream,
    Source,
    Decoder
};
use std::{
    fs::{ File, create_dir_all, metadata },
    io::{ Error, Write },
    os::unix::net::{ UnixListener, UnixStream },
    sync::{ Arc, Mutex },
    env::var,
    path::PathBuf,
    time::Duration,
    thread,
};

// TODO:
// daemonize command
// Add playlist feature
// Add indexing crate
// update mpd controls
// play-pause
// -t can queue song & change song

/// Plays songs in the background of your desktop
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Song filepath
    #[arg(short, long, default_value = "")]
    title: String,

    /// Playlist path/title
    #[arg(short, long, default_value = "")]
    playlist: String,

    /// 1 = Play, 2 = Pause, 3 = Stop, 4 = Change song, 5 = Queue song
    #[arg(short, long, default_value_t = 0)] // if 0 do nothing
    command: u8,

    /// 1 = Loop; 0 = No Loop
    #[arg(short, long, default_value_t = 0)]
    _loop: u8
}

fn daemonize() -> bool {
    let homedir = var("HOME").expect("Could not find $HOME environment variable");
    let playmedir = PathBuf::from(format!("{}/.local/share/playmectl/", homedir));

    if !playmedir.exists() {
        create_dir_all(&playmedir).expect("Failed to create playmectl directory");
    }

    let stdout = File::create(playmedir.join("daemon.out")).unwrap();
    let stderr = File::create(playmedir.join("daemon.err")).unwrap();

    let daemonize = Daemonize::new()
        .pid_file(playmedir.join("playmectl.pid"))
        .working_directory(playmedir)
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn event_handler() {}

fn filepath_exists(file_path: &str) -> u8 {
    if let Ok(meta) = metadata(file_path) {
        if meta.is_file() {
            return 1;
        } else if meta.is_dir() {
            return 2;
        }
    }

    0
}

fn manage_audio(current_song: &mut Sink, opt: u8) {
    match opt {
        0 => { // play/pause
            if current_song.is_paused() {
                current_song.play();
                println!("Resumed");
            } else {
                current_song.pause();
                println!("Paused");
            }
        }

        1 => { // Manual Queue

        }

        _ => println!("ERROR")
    }
}

fn play_audio(path_type: u8, file_path: &str, _loop: u8) {
    match path_type {
        1 => {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();

            let song = std::fs::File::open(file_path).unwrap();
            let source = Decoder::new(std::io::BufReader::new(song)).unwrap();

            if _loop == 1 {
                let infinite_source = source.repeat_infinite();
                sink.append(infinite_source);
                loop {
                    sink.sleep_until_end();
                    thread::sleep(Duration::from_secs(3));
                }
            } else {
                sink.append(source);
                while !sink.empty() {
                    thread::sleep(Duration::from_millis(100));
                }
                println!("playmectl: {} has completed...", file_path);
            }

            sink.stop();
        }

        2 => { // PlayList code :)

        }

        _ => eprintln!("ERROR")
    }
}

fn main() {
    daemonize();
    let args = Args::parse();

    if !args.title.is_empty() {
        let path_result: u8 = filepath_exists(args.title.as_str());
        if path_result != 0 {
            play_audio(path_result, &args.title, args._loop);
        } else {
            eprintln!("File '{}' does not exist.", args.title);
        }
    }

    if !args.playlist.is_empty() {
        println!("{}", args.playlist);
    }

    eprintln!("Incorrect arguments supplied\nPlease try again");
}
