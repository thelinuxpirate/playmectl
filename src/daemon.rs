use crate::DirData;
use daemonize::Daemonize;
use std::{
    os::unix::net::{ UnixListener, UnixStream },
    io::{ Read, Write, Error },
    fs::{ create_dir_all, metadata, remove_file, File },
};

pub fn daemonize() -> bool {
    let dirs = DirData::new();

    if !dirs.playmedir.exists() { // remove ensure_dirs from Struct?
            create_dir_all(&dirs.playmedir).expect("Failed to create playmectl directory");
    }

    let stdout = File::create(dirs.playmedir.join("daemon.out")).unwrap();
    let stderr = File::create(dirs.playmedir.join("daemon.err")).unwrap();

    let daemonize = Daemonize::new()
        .pid_file(dirs.playmedir.join("playmectl.pid"))
        .working_directory(dirs.playmedir)
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn start_socket() -> Result<UnixListener, Error> {
    let dirs = DirData::new();

    if metadata(&dirs.socket_path).is_ok() {
        remove_file(&dirs.socket_path).unwrap();
        println!("Terminated old socket...\nREMOVED: {}", &dirs.socket_path.display());
    }

    let listener = UnixListener::bind(&dirs.socket_path).expect("Failed to bind socket");
    println!("Server listening on {}", &dirs.socket_path.display());

    Ok(listener)
}

pub fn socket_manager(listener: UnixListener) -> std::io::Result<()> {
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Client connected!");
                if let Err(e) = handle_client(&mut stream) {
                    eprintln!("Client handling error: {}", e);
                }
            }
            Err(err) => eprintln!("Connection failed: {}", err),
        }
    }
    Ok(())
}


pub fn handle_client(stream: &mut UnixStream) -> std::io::Result<()> {
    let mut buffer = [0; 128];
    let size = stream.read(&mut buffer)?;

    if size == 0 {
        return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Client disconnected"));
    }

    let message = String::from_utf8_lossy(&buffer[..size]);
    println!("Received: {}", message);

    // Send response
    stream.write_all(b"Hello from server!\n")?;
    Ok(())
}
