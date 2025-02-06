use clap::Parser;
use daemonize::Daemonize;
use rodio::{
    Sink,
    OutputStream,
    Source,
    Decoder
};
use std::{
    fs::{ File, create_dir_all, metadata, read_to_string },
    io::{ BufReader, Write },
//    sync::{ Arc, Mutex },
    env::var,
    path::PathBuf,
    time::Duration,
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
    view: bool
}

struct AudioManager {
    sink: Sink
}

impl AudioManager {
    fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        AudioManager { sink }
    }

    fn manage_audio(&self, path_type: Option<u8>, file_path: &str, cmd: u8) {
        let song = File::open(file_path).unwrap();
        let source = Decoder::new(BufReader::new(song)).unwrap();

        update_currently_playing(file_path);

        match path_type {
            Some(1) => {
                if cmd == 3 {
                    let infinite_source = source.repeat_infinite();
                    self.sink.append(infinite_source);
                    loop {
                        self.sink.sleep_until_end();
                        thread::sleep(Duration::from_secs(3));
                    }
                } else {
                    self.sink.append(source);
                    while !self.sink.empty() {
                        thread::sleep(Duration::from_millis(100));
                    }
                    println!("playmectl: {} has completed...", file_path);
                }

                self.sink.stop();
                update_currently_playing("");
            }

            Some(2) => { // PlayList code :)
                // Implement playlist handling here
            }

            _ => eprintln!("Now how did we get here?"),
        }

        match cmd {
            1 => {
                if self.sink.is_paused() {
                    self.sink.play();
                    println!("Resumed");
                } else {
                    self.sink.pause();
                    println!("Paused");
                }
            }

            2 => {
                self.sink.stop();
                println!("Process has been killed");
            }

            _ => println!("Unknown command given\nPlease read the 'man' page"),
        }
    }
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

fn filepath_exists(file_path: &str) -> Option<u8> {
    if let Ok(meta) = metadata(file_path) {
        if meta.is_file() {
            return Some(1);
        } else if meta.is_dir() {
            return Some(2);
        }
    }

    Some(0)
}

fn update_currently_playing(song: &str) {
    let homedir = var("HOME").expect("Could not find $HOME environment variable");
    let playmedir = PathBuf::from(format!("{}/.local/share/playmectl/", homedir));
    let state_file = playmedir.join("currently_playing.txt");

    let mut file = File::create(state_file).expect("Could not create state file");
    writeln!(file, "{}", song).expect("Could not write to state file");
}

fn get_currently_playing() -> Option<String> {
    let homedir = var("HOME").expect("Could not find $HOME environment variable");
    let playmedir = PathBuf::from(format!("{}/.local/share/playmectl/", homedir));
    let state_file = playmedir.join("currently_playing.txt");

    if let Ok(contents) = read_to_string(state_file) {
        let path = PathBuf::from(contents.trim());
        if let Some(filename) = path.file_name() {
            return Some(filename.to_string_lossy().to_string());
        }
    }
    None
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
        let path_result = filepath_exists(args.title.as_str());
        if path_result != Some(0) {
            // queuing logic would go HERE


            daemonize();
            let audio_manager = AudioManager::new(); // logic error?
            audio_manager.manage_audio(Some(path_result).expect("Failed to read path"), &args.title, args.command);
        } else {
            eprintln!("File '{}' does not exist.", args.title);
        }
    }

    if args.command != 0 {
        let audio_manager = AudioManager::new();
        audio_manager.manage_audio(None, &args.title, args.command);
    }

    eprintln!("Incorrect arguments supplied\nPlease try again");
}
