use std::{
    ffi::OsStr,
    fs::{read_dir, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use rand::seq::SliceRandom;
use rodio::{Decoder, OutputStreamHandle, Sink};

pub struct RMPlayer {
    queue: Vec<PathBuf>,
    current: usize,
    sink: Sink,
    _shuffle: bool,
    infinite: bool,
    path: String,
}

// private methods
impl RMPlayer {
    // append a file to the sink
    fn add_file_to_sink(&self, filepath: &str) {
        let file = BufReader::new(File::open(filepath).unwrap());
        let source = Decoder::new(file).unwrap();

        self.sink.append(source); // lock and append source
    }
}

impl RMPlayer {
    pub fn new(
        stream_handle: OutputStreamHandle,
        path: String,
        shuffle: bool,
        infinite: bool,
    ) -> Self {
        let sink = Sink::try_new(&stream_handle).unwrap();
        let queue = get_folder_files(Path::new(&path), true).unwrap();

        let queue = match shuffle {
            true => shuffle_vec(queue),
            false => queue,
        };

        Self {
            queue,
            current: 0,
            sink,
            _shuffle: shuffle,
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
    }

    pub fn play_pause(&self) {
        match self.sink.is_paused() {
            true => self.sink.play(),
            false => self.sink.pause(),
        };
    }

    pub fn next(&mut self) {
        self.sink.skip_one();

        self.current = if self.current >= self.queue.len() {
            0
        } else {
            self.current + 1
        };
    }

    pub fn volume_up(&self) {
        let volume = self.sink.volume() + 0.1;
        let volume = if volume >= 1.5 { 1.5 } else { volume };

        self.sink.set_volume(volume);
    }

    pub fn volume_down(&self) {
        let volume = self.sink.volume() - 0.1;
        let volume = if volume <= 0.0 { 0.0 } else { volume };

        self.sink.set_volume(volume);
    }

    // TODO: save current element (self.queue.get(self.current))
    // shuffle all the others and then put back the current element
    // in the first position
    pub fn shuffle_playlist(&mut self) {
        self.queue = shuffle_vec(self.queue.clone());
    }

    pub fn get_sink(&self) -> &Sink {
        &self.sink
    }

    pub fn get_sink_len(&self) -> usize {
        self.sink.len()
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn get_infinte(&self) -> bool {
        self.infinite
    }

    pub fn get_current_filename(&self) -> &OsStr {
        let cur = self.queue.get(self.current).unwrap();

        let cur_name = cur.file_name().unwrap();
        cur_name
    }

    pub fn get_volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn set_current(&mut self, value: usize) -> usize {
        self.current = value;
        self.current
    }
}

fn shuffle_vec<T>(vec: Vec<T>) -> Vec<T> {
    let mut rng = rand::thread_rng();
    let mut vec = vec;

    vec.shuffle(&mut rng);

    vec
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
