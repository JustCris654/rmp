use std::{
    collections::VecDeque,
    fs::{metadata, read_dir, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use rodio::{Decoder, OutputStream, Sink};

pub struct RMPlayer {
    queue: VecDeque<PathBuf>,
    sink: Arc<Mutex<Sink>>,
    shuffle: bool,
    infinite: bool,
    path: String,
}

impl RMPlayer {
    pub fn new(&self, path: String, shuffle: bool, infinite: bool) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle).unwrap()));

        Self {
            queue: VecDeque::new(),
            sink,
            shuffle,
            infinite,
            path,
        }
    }
}

// append a file to the sink
fn add_file_to_sink(sink: MutexGuard<Sink>, filepath: &str) {
    let file = BufReader::new(File::open(filepath).unwrap());
    let source = Decoder::new(file).unwrap();

    sink.append(source);
}

// get files paths in a folder, if rec is true search recursively in the subfolders, and add them in a VecDeque
// only flac, wav and mp3 files are added to the VecDeque
fn get_folder_files(folder: &Path, rec: bool) -> io::Result<VecDeque<PathBuf>> {
    let paths = read_dir(folder).unwrap();

    let mut files: VecDeque<PathBuf> = VecDeque::new();

    for path in paths {
        let path = path.unwrap().path();

        if path.is_dir() && rec {
            files.extend(get_folder_files(&path, rec)?);
        } else if let Some(ext) = path.extension() {
            match ext.to_str().unwrap() {
                "flac" => files.push_back(path),
                "wav" => files.push_back(path),
                "mp3" => files.push_back(path),
                _ => {}
            }
        }
    }

    Ok(files)
}
