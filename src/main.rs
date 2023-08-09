use clap::Parser;
use crossterm::event::{read, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use rodio::{Decoder, OutputStream, Sink};
use std::collections::VecDeque;
use std::fs::File;
use std::fs::{metadata, read_dir};
use std::io::{self, stdout, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

#[derive(Parser)]
struct Args {
    path: Option<String>,
    #[arg(short = 's', long = "shuffle")]
    shuffle: bool,
}

fn add_file_to_sink(sink: MutexGuard<Sink>, filepath: &str) {
    let file = BufReader::new(File::open(filepath).unwrap());
    let source = Decoder::new(file).unwrap();

    sink.append(source);
}

// get files paths in a folder, if rec is true search recursively in the subfolders, and add them in a queue
fn get_folder_files(folder: &Path, rec: bool) -> io::Result<Vec<PathBuf>> {
    let paths = read_dir(folder).unwrap();

    let mut files: Vec<PathBuf> = Vec::new();

    for path in paths {
        let path = path.unwrap().path();

        if path.is_dir() && rec {
            files.extend(get_folder_files(&path, rec)?);
        } else {
            files.push(path);
        }
    }

    Ok(files)
}

fn main() {
    let args = Args::parse();
    let filepath = args.path.unwrap();

    let filepath = Path::new(&filepath);

    let md = metadata(filepath).unwrap();

    let infinite = true;

    assert!(
        !md.is_file(),
        "Not implemented -> start with file and continue with music saved in db"
    );

    // get all files in the folder, this queue will not be used directly
    let folder_files: Vec<PathBuf> = get_folder_files(filepath, false).unwrap();

    // get an output stream to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle).unwrap()));

    // append to the sink all files in the queue
    let _ = folder_files
        .iter()
        .map(|file| {
            add_file_to_sink(
                sink.clone().lock().unwrap(),
                file.as_path().to_str().unwrap(),
            )
        })
        .collect::<Vec<_>>();

    enable_raw_mode().unwrap();

    // clear terminal
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All)).unwrap();

    // checks every 1 second if the sink has less than one file in the queue, if true
    // and infinite flag is set reappend the queue of files in the sink
    // if false do nothing
    let sh_sink = Arc::clone(&sink);
    let _sink_handler = thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_secs(1));

        println!("sink len: {}", sh_sink.lock().unwrap().len());

        let sink_len = { sh_sink.lock().unwrap().len() };

        if sink_len <= 1 && infinite {
            let _ = folder_files
                .iter()
                .map(|file| {
                    add_file_to_sink(sh_sink.lock().unwrap(), file.as_path().to_str().unwrap())
                })
                .collect::<Vec<_>>();
        }
    });

    let ih_sink = Arc::clone(&sink);
    let input_handler = thread::spawn(move || {
        loop {
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => {
                        let sink = ih_sink.lock().unwrap();
                        if sink.is_paused() {
                            sink.play();
                        } else {
                            sink.pause();
                        }
                    }
                    KeyCode::Char('s') => {
                        // shuffle
                    }
                    KeyCode::Char('n') | KeyCode::Char('l') => {
                        // next music in track list

                        let sink = ih_sink.lock().unwrap();
                        sink.skip_one();
                    }
                    KeyCode::Char('p') | KeyCode::Char('h') => {
                        // previous in track list
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        // volume up
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        // volume down
                    }
                    _ => {}
                }
            }
        }
    });

    input_handler.join().unwrap();

    disable_raw_mode().unwrap();
    println!("Raw mode disabled");
}
