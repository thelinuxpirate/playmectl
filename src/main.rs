use clap::Parser;
use playmectl::{
    daemon::{
        daemonize,
        start_socket,
        socket_manager
    },
    get_currently_playing,
    update_currently_playing,
    DirData
};
use rodio::{
    Sink,
    OutputStream,
    Source,
    Decoder
};
use std::{
    time::Duration,
    io::BufReader,
    fs::File,
    thread,
};

// TODO:
// Add playlist feature
// Add indexing crate
// update mpd controls
// add libnotify options
// add manual queuing with -t & -c (rodio 'append')
// -t can queue song & change song

/// Plays songs in the background of your desktop
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Playlist/Song filepath
    #[arg(short, long, default_value = "")]
    title: String,

    /// 1 = play/pause, 2 = kill, 3 = loop,  4 = Change song, 5 = Queue song
    #[arg(short, long, default_value_t = 0)] // if 0 do nothing
    command: u8,

    /// View currently playing song
    #[arg(short, long)]
    view: bool,

    /// test me
    #[arg(short, long, default_value_t = 0)]
    demo: u8

}

struct AudioManager {
    sink: Sink,
    status: bool
}

impl AudioManager {
    fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let status = true;
        AudioManager { sink, status }
    }

    fn get_status(am: AudioManager) -> bool {
        if am.status == false {
            false
        } else {
            true
        }
    }
}

fn manage_audio(am: AudioManager, path_type: Option<u8>, file_path: &str, cmd: u8) {
    let song = File::open(file_path).unwrap();
    let source = Decoder::new(BufReader::new(song)).unwrap();

    update_currently_playing(file_path);

    match path_type {
        Some(1) => {
            if cmd == 3 {
                let infinite_source = source.repeat_infinite();
                am.sink.append(infinite_source);
                loop {
                    am.sink.sleep_until_end();
                    thread::sleep(Duration::from_secs(3));
                }
            } else {
                am.sink.append(source);
                while !am.sink.empty() {
                    thread::sleep(Duration::from_millis(100));
                }
                println!("playmectl: {} has completed...", file_path);
            }

            am.sink.stop();
            update_currently_playing("");
        }

        Some(2) => { // PlayList code :)
            // Implement playlist handling here
        }

        _ => eprintln!("Now how did we get here?"),
    }

    match cmd {
        1 => {
            if am.sink.is_paused() {
                am.sink.play();
                println!("Resumed");
            } else {
                am.sink.pause();
                println!("Paused");
            }
        }

        2 => {
            am.sink.stop();
            println!("Process has been killed");
        }

        _ => println!("Unknown command given\nPlease read the 'man' page"),
    }
}

fn send_cmd(cmd: u8) {
    match cmd {



        _ => println!("fuck"),
    }


}

fn main() {
    let args = Args::parse();

    if args.view {
        if let Some(current_song) = get_currently_playing() {
            if current_song.is_empty() {
                eprintln!("There is nothing currently playing!\nTry using the -t command");
            } else {
                println!("Currently playing: '{}'", current_song);
            }
        } else {
            eprintln!("Failed to retrieve currently playing song");
        }
        return;
    }

    if !args.title.is_empty() {
        let path_result = DirData::filepath_exists(args.title.as_str());
        if path_result != Some(0) {
            daemonize();
            let audio_manager = AudioManager::new();
            manage_audio(audio_manager, Some(path_result).expect("Failed to read path"), &args.title, args.command);
        } else {
            eprintln!("File '{}' does not exist.", args.title);
        }
    }

    if args.command != 0 {
        let audio_manager = AudioManager::new();
        manage_audio(audio_manager, None, &args.title, args.command);
    }

    if args.demo != 0 {
        daemonize();
        match start_socket() {
            Ok(listener) => {
                if let Err(e) = socket_manager(listener) {
                    eprintln!("Socket manager error: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to start socket: {}", e),
        }
    }

    eprintln!("Incorrect arguments supplied\nPlease try again");
}
