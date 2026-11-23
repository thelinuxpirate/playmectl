use clap::Parser;
use playmectl::{
    daemon::{daemonize, socket_manager, start_socket},
    get_currently_playing, AudioManager, DirData,
};
use std::{io::Write, os::unix::net::UnixStream, process::exit};

/// Plays songs in the background of your desktop
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Playlist/Song filepath
    #[arg(short, long, default_value = "")]
    title: String,

    /// 1 = play/pause, 2 = kill, 3 = loop,  4 = Change song, 5 = Queue song
    #[arg(short, long, default_value = "append")]
    command: String,

    /// View currently playing song
    #[arg(short, long)]
    view: bool,

    /// Send desktop notifications when tracks change
    #[arg(short = 'n', long)]
    notify: bool,
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
        if !args.title.is_empty() {
            let command = match args.command.as_str() {
                "5" | "queue" | "queue_track" => "queue_track",
                _ => "change_track",
            };
            send_cmd(&format!("{} {}", command, args.title));
            return;
        }

        match args.command.as_str() {
            "0" | "append" => send_cmd("append"),
            "1" | "toggle" | "play_pause" => send_cmd("toggle_play"),
            "2" | "kill" | "stop" => send_cmd("kill"),
            "3" | "loop" | "infinite" => send_cmd("make_infinite"),
            "4" | "next" => send_cmd("next_track"),
            "5" | "queue" | "queue_track" if args.title.is_empty() => send_cmd("queue_track "),
            "6" | "previous" | "prev" => send_cmd("previous_track"),
            "7" | "pause" => send_cmd("pause"),
            "8" | "play" => send_cmd("play"),
            _ => eprintln!("Unknown command."),
        }
        return;
    }

    if !args.title.is_empty() {
        daemonize();
        let mut audio_manager = AudioManager::new_with_options(args.title, args.notify);

        match start_socket() {
            Ok(listener) => {
                if let Err(e) = socket_manager(listener, &mut audio_manager) {
                    eprintln!("Socket manager error: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to start socket: {}", e),
        }
    }

    eprintln!("Invalid arguments. Try using `-h`, `-t <song>`, or `-c <command>`");
}
