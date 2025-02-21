use clap::Parser;
use playmectl::{
    daemon::{
        daemonize,
        start_socket,
        socket_manager
    },
    get_currently_playing,
    AudioManager,
    DirData
};
use std::{
    os::unix::net::UnixStream,
    process::exit,
    io::Write,
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
pub struct Args {
    /// Playlist/Song filepath
    #[arg(short, long, default_value = "")]
    title: String,

    /// 1 = play/pause, 2 = kill, 3 = loop,  4 = Change song, 5 = Queue song
    #[arg(short, long, default_value_t = 0)]
    command: u8,

    /// View currently playing song
    #[arg(short, long)]
    view: bool,

}

fn send_cmd(command: &str) {
    let dirs = DirData::new();
    if let Ok(mut stream) = UnixStream::connect(&dirs.socket_path) {
        stream.write_all(command.as_bytes()).unwrap();
    } else {
        eprintln!("Error: Daemon is not running. Start with `-t <song>`");
        exit(1);
    }
}

fn main() {
    let args = Args::parse();
    let dirs = DirData::new();

    if args.view {
        if let Some(current_song) = get_currently_playing() {
            if current_song.is_empty() {
                eprintln!("Nothing is currently playing.");
            } else {
                println!("Currently playing: {}", current_song);
            }
        }
        return;
    }

    if dirs.socket_path.exists() {
        match args.command {
            0 => send_cmd("append"),
            1 => send_cmd("toggle_play"),
            2 => send_cmd("stop"),
            3 => send_cmd("infinite"),
            _ => eprintln!("Unknown command."),
        }
        return;
    }

    if !args.title.is_empty() {
        daemonize();
        let mut audio_manager = AudioManager::new(args.title);

        match start_socket() {
            Ok(listener) => {
                if let Err(e) = socket_manager(listener, &mut audio_manager) {
                    eprintln!("Socket manager error: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to start socket: {}", e),
        }
    }

    eprintln!("Invalid arguments. Try using `-t <song>` or `-c <command>`");
}
