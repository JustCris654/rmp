use std::{
    fs::{metadata, read_dir, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

pub struct RMPlayer {
    queue: Vec<PathBuf>,
    current: usize,
    sink: Arc<Mutex<Sink>>,
    shuffle: bool,
    infinite: bool,
    path: String,
}

// private methods
impl RMPlayer {
    // append a file to the sink
    fn add_file_to_sink(&self, filepath: &str) {
        let file = BufReader::new(File::open(filepath).unwrap());
        let source = Decoder::new(file).unwrap();

        let sink = self.sink.clone(); // get arc reference
        sink.lock().unwrap().append(source); // lock and append source
    }
}

impl RMPlayer {
    pub fn new(
        stream_handle: OutputStreamHandle,
        path: String,
        shuffle: bool,
        infinite: bool,
    ) -> Self {
        let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle).unwrap()));
        let queue = get_folder_files(Path::new(&path), true).unwrap();

        Self {
            queue,
            current: 0,
            sink,
            shuffle,
            infinite,
            path,
        }
    }

    pub fn fill_sink(&self) {
        let _ = self
            .queue
            .iter()
            .map(|file| self.add_file_to_sink(file.as_path().to_str().unwrap()))
            .collect::<Vec<_>>();

        self.sink.lock().unwrap().play();
    }

    pub fn get_sink(&self) -> &Arc<Mutex<Sink>> {
        &self.sink
    }
}

// get files paths in a folder, if rec is true search recursively in the subfolders, and add them in a vector
// only flac, wav and mp3 files are added to the vector
fn get_folder_files(folder: &Path, rec: bool) -> io::Result<Vec<PathBuf>> {
    let paths = read_dir(folder).unwrap();

    let mut files: Vec<PathBuf> = Vec::new();

    for path in paths {
        let path = path.unwrap().path();

        if path.is_dir() && rec {
            files.extend(get_folder_files(&path, rec)?);
        } else if let Some(ext) = path.extension() {
            match ext.to_str().unwrap() {
                "flac" => files.push(path),
                "wav" => files.push(path),
                "mp3" => files.push(path),
                _ => {}
            }
        }
    }

    Ok(files)
}
