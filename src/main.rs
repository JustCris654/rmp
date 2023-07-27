use clap::Parser;
use crossterm::event::{read, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::{stdout, BufReader};
use std::thread;

#[derive(Parser)]
struct Args {
    path: Option<String>,
    #[arg(short = 's', long = "shuffle")]
    shuffle: bool,
}

fn main() {
    let args = Args::parse();
    let filepath = args
        .path
        .unwrap_or("./media/02 - Universally Speaking.flac".to_string());

    // get an output stream to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let sink = Sink::try_new(&stream_handle).unwrap();

    let file = BufReader::new(File::open(filepath).unwrap());
    let source = Decoder::new(file).unwrap();

    sink.append(source);

    enable_raw_mode().unwrap();

    // clear terminal
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All)).unwrap();

    let input_handler = thread::spawn(move || loop {
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
                _ => {}
            }
        }
    });

    input_handler.join().unwrap();

    disable_raw_mode().unwrap();
    println!("Raw mode disabled");
}
