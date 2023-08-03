use clap::Parser;
use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{execute, style::Print};
use rodio::{Decoder, OutputStream, Sink};
use std::collections::VecDeque;
use std::fs::File;
use std::fs::{metadata, read_dir};
use std::io::{self, stdout, BufReader};
use std::path::{Path, PathBuf};
use std::thread;

#[derive(Parser)]
struct Args {
    path: Option<String>,
    #[arg(short = 's', long = "shuffle")]
    shuffle: bool,
}

fn add_file_to_sink(sink: &Sink, filepath: &str) {
    let file = BufReader::new(File::open(filepath).unwrap());
    let source = Decoder::new(file).unwrap();

    sink.append(source);
}

// get files paths in a folder, if rec is true search recursively in the subfolders, and add them in a queue
fn get_folder_files(folder: &Path, rec: bool) -> io::Result<VecDeque<PathBuf>> {
    let paths = read_dir(folder).unwrap();

    let mut files: VecDeque<PathBuf> = VecDeque::new();

    for path in paths {
        let path = path.unwrap().path();

        if path.is_dir() && rec {
            files.extend(get_folder_files(&path, rec)?);
        } else {
            files.push_front(path);
        }
    }

    Ok(files)
}

fn main() {
    let args = Args::parse();
    let filepath = args.path.unwrap();

    let filepath = Path::new(&filepath);

    let md = metadata(filepath).unwrap();

    assert!(
        !md.is_file(),
        "Not implemented -> start with file and continue with music saved in db"
    );

    let mut queue: VecDeque<PathBuf> = get_folder_files(filepath, false).unwrap();

    // get an output stream to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let sink = Sink::try_new(&stream_handle).unwrap();

    add_file_to_sink(&sink, queue.pop_back().unwrap().as_path().to_str().unwrap());

    enable_raw_mode().unwrap();

    // clear terminal
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All)).unwrap();

    let input_handler = thread::spawn(move || {
        loop {
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => {
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
                        execute!(std::io::stdout(), Print("Skipping a song\n".to_string()))
                            .unwrap();
                        sink.clear();

                        add_file_to_sink(
                            &sink,
                            queue.pop_back().unwrap().as_path().to_str().unwrap(),
                        );

                        sink.play();
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
