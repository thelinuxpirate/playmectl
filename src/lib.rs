use rodio::{OutputStream, Sink};
use std::{
    collections::VecDeque,
    env::var,
    fs::{metadata, read_to_string, File},
    io::Write,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub mod daemon;

pub struct AudioManager {
    pub track: String,
    pub _stream: OutputStream,
    pub sink: Sink,
    pub status: bool,
    pub queue: VecDeque<String>,
    pub history: VecDeque<String>,
    pub notifications: bool,
}

impl AudioManager {
    pub fn new(track: String) -> Self {
        Self::new_with_options(track, false)
    }

    pub fn new_with_options(track: String, notifications: bool) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let status = true;
        let mut resolved = resolve_media_paths(&track);
        let track = resolved.pop_front().unwrap_or(track);
        AudioManager {
            track,
            _stream,
            sink,
            status,
            queue: resolved,
            history: VecDeque::new(),
            notifications,
        }
    }

    // testing function
    pub fn get_status(am: &Self) -> bool {
        if am.sink.empty() {
            true
        } else {
            false
        }
    }
}

pub struct Playlist {
    pub paths: Vec<PathBuf>,
}

impl Playlist {
    pub fn load(path: &Path) -> Option<Self> {
        if path.is_dir() {
            return Some(Self {
                paths: resolve_media_paths(path.to_string_lossy().as_ref())
                    .into_iter()
                    .map(PathBuf::from)
                    .collect(),
            });
        }

        let extension = path.extension()?.to_string_lossy().to_lowercase();
        if matches!(extension.as_str(), "m3u" | "m3u8" | "pls") {
            let mut paths = Vec::new();
            if let Ok(contents) = read_to_string(path) {
                let parent = path.parent().unwrap_or_else(|| Path::new("."));
                for line in contents
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                {
                    let candidate = if line.contains('=') && extension == "pls" {
                        line.split_once('=').map(|(_, value)| value).unwrap_or(line)
                    } else {
                        line
                    };
                    let candidate = PathBuf::from(candidate);
                    let resolved = if candidate.is_absolute() {
                        candidate
                    } else {
                        parent.join(candidate)
                    };
                    if resolved.is_file() {
                        paths.push(resolved);
                    }
                }
            }
            if !paths.is_empty() {
                return Some(Self { paths });
            }
        }
        None
    }
}

pub struct DirData {
    playmedir: PathBuf,
    pub socket_path: PathBuf,
    state_file: PathBuf,
    pub pid_file: PathBuf,
}

impl DirData {
    pub fn new() -> Self {
        let homedir = var("HOME").expect("Could not find $HOME environment variable");
        let playmedir = PathBuf::from(format!("{}/.local/share/playmectl/", homedir));
        let socket_path =
            PathBuf::from(format!("{}/.local/share/playmectl/playme.socket", homedir));
        let state_file = playmedir.join("currently_playing.txt");
        let pid_file = playmedir.join("playmectl.pid");

        Self {
            playmedir,
            socket_path,
            state_file,
            pid_file,
        }
    }

    // Playlist code
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

pub fn is_supported_media(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()).map(|extension| extension.to_lowercase()),
        Some(extension) if matches!(extension.as_str(), "mp3" | "wav" | "ogg" | "flac" | "aac" | "m4a" | "opus")
    )
}

pub fn resolve_media_paths(input: &str) -> VecDeque<String> {
    let path = PathBuf::from(input);
    if path.is_file() {
        if is_supported_media(&path) {
            return VecDeque::from([path.to_string_lossy().to_string()]);
        }
        if let Some(playlist) = Playlist::load(&path) {
            return playlist
                .paths
                .into_iter()
                .map(|path| path.to_string_lossy().to_string())
                .collect();
        }
    }
    if path.is_dir() {
        let mut paths = Vec::new();
        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file() && is_supported_media(entry.path()) {
                paths.push(entry.path().to_path_buf());
            }
        }
        paths.sort();
        return paths
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();
    }
    VecDeque::new()
}

pub fn notify_track(track: &str) {
    if let Some(name) = Path::new(track).file_name() {
        let _ = notify_rust::Notification::new()
            .summary("Now playing")
            .body(&name.to_string_lossy())
            .show();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_dir(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("playmectl-{name}-{id}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn indexes_supported_files_in_directory() {
        let dir = temp_dir("directory");
        fs::write(dir.join("b.mp3"), []).unwrap();
        fs::write(dir.join("a.flac"), []).unwrap();
        fs::write(dir.join("ignored.txt"), []).unwrap();

        let tracks = resolve_media_paths(dir.to_string_lossy().as_ref());
        assert_eq!(tracks.len(), 2);
        assert!(tracks
            .iter()
            .all(|track| is_supported_media(Path::new(track))));
    }

    #[test]
    fn loads_relative_m3u_entries() {
        let dir = temp_dir("playlist");
        let track = dir.join("song.wav");
        fs::write(&track, []).unwrap();
        let playlist = dir.join("queue.m3u");
        fs::write(&playlist, "#EXTM3U\nsong.wav\n").unwrap();

        let tracks = resolve_media_paths(playlist.to_string_lossy().as_ref());
        assert_eq!(tracks.len(), 1);
        assert_eq!(PathBuf::from(tracks.front().unwrap()), track);
    }
}
