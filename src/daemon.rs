use crate::{AudioManager, DirData};
use daemonize::Daemonize;
use rodio::Decoder;
use std::{
    fs::{create_dir_all, metadata, remove_file, File},
    io::{Read, BufReader},
    os::unix::net::UnixListener,
    process::exit,
};

pub fn daemonize() {
    let dirs = DirData::new();
    if !dirs.playmedir.exists() {
        create_dir_all(&dirs.playmedir).expect("Failed to create playmectl directory");
    }

    let stdout = File::create(dirs.playmedir.join("daemon.out")).unwrap();
    let stderr = File::create(dirs.playmedir.join("daemon.err")).unwrap();

    let daemonize = Daemonize::new()
        .pid_file(dirs.playmedir.join("playmectl.pid"))
        .working_directory(dirs.playmedir.clone())
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => println!("Daemon started"),
        Err(_) => exit(1),
    }
}

pub fn start_socket() -> std::io::Result<UnixListener> {
    let dirs = DirData::new();

    if metadata(&dirs.socket_path).is_ok() {
        remove_file(&dirs.socket_path).unwrap();
    }

    let listener = UnixListener::bind(&dirs.socket_path)?;
    println!("Server listening on socket: {}", dirs.socket_path.display());
    Ok(listener)
}

pub fn socket_manager(
    listener: UnixListener,
    audio_manager: &mut AudioManager,
) -> std::io::Result<()> {
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 128];
                let size = stream.read(&mut buffer)?;

                if size == 0 {
                    return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Client disconnected"));
                }

                let message = String::from_utf8_lossy(&buffer[..size]).trim().to_string();
                println!("Received command: {}", message);

                match message.as_str() {
                    "append" => {
                        match File::open(&audio_manager.track) {
                            Ok(song) => {
                                let source = Decoder::new(BufReader::new(song)).unwrap();
                                audio_manager.sink.append(source);
                                println!("HERE");
                            }
                            Err(err) => {
                                eprintln!("Failed to open file: {}", err);
                            }
                        }
                    },
                    "play" => audio_manager.sink.play(),
                    "pause" => audio_manager.sink.pause(),
                    "stop" => audio_manager.sink.stop(),
                    _ => eprintln!("Unknown command"),
                }
            }
            Err(err) => eprintln!("Connection failed: {}", err),
        }
    }
    Ok(())
}
