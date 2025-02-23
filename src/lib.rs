use std::{
    fs::{
        File,
        metadata,
        read_to_string
    },
    path::PathBuf,
    io::Write,
    env::var,
};
use rodio::{
    Sink,
    OutputStream,
};

pub mod daemon;

pub struct AudioManager {
    pub track: String,
    pub _stream: OutputStream,
    pub sink: Sink,
    pub status: bool
}

impl AudioManager {
    pub fn new(track: String) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let status = true;
        AudioManager { track, _stream, sink, status }
    }

    pub fn get_status(am: &Self) -> bool {
        if am.sink.empty() {
            true
        } else {
           false
        }
    }
}

pub struct DirData {
    playmedir: PathBuf,
    pub socket_path: PathBuf,
    state_file: PathBuf,
    pub pid_file: PathBuf
}

impl DirData {
    pub fn new() -> Self {
        let homedir = var("HOME").expect("Could not find $HOME environment variable");
        let playmedir = PathBuf::from(format!("{}/.local/share/playmectl/", homedir));
        let socket_path = PathBuf::from(format!("{}/.local/share/playmectl/playme.socket", homedir));
        let state_file = playmedir.join("currently_playing.txt");
        let pid_file = playmedir.join("playmectl.pid");

        Self {
            playmedir,
            socket_path,
            state_file,
            pid_file
        }
    }

    pub fn filepath_exists(file_path: &str) -> Option<u8> {
        if let Ok(meta) = metadata(file_path) {
            if meta.is_file() {
                return Some(1);
            } else if meta.is_dir() {
                return Some(2);
            }
        }

        Some(0)
    }
}

pub fn update_currently_playing(song: &str) {
    let dirs = DirData::new();

    let mut file = File::create(dirs.state_file).expect("Could not create state file");
    writeln!(file, "{}", song).expect("Could not write to state file");
}

pub fn get_currently_playing() -> Option<String> {
    let dirs = DirData::new();

    if let Ok(contents) = read_to_string(dirs.state_file) {
        let path = PathBuf::from(contents.trim());
        if let Some(filename) = path.file_name() {
            return Some(filename.to_string_lossy().to_string());
        }
    }
    None
}
