use clap::Parser;
use crossterm::event::{read, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use rmp::RMPlayer;
use rodio::OutputStream;
use std::fs::metadata;
use std::io::stdout;
use std::path::Path;
use std::sync::Arc;
use std::thread;

#[derive(Parser)]
struct Args {
    path: Option<String>,
    #[arg(short = 's', long = "shuffle")]
    shuffle: bool,
    infinite: bool,
}

fn main() {
    let args = Args::parse();
    let filepath = args.path.unwrap();

    let filepath = Path::new(&filepath);

    let md = metadata(filepath).unwrap();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let rmplayer = Arc::new(RMPlayer::new(
        stream_handle.clone(),
        filepath.to_str().unwrap().to_string(),
        args.shuffle,
        args.infinite,
    ));

    assert!(
        !md.is_file(),
        "Not implemented -> start with file and continue with music saved in db"
    );

    rmplayer.clone().fill_sink();

    enable_raw_mode().unwrap();

    // clear terminal
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All)).unwrap();

    // checks every 1 second if the sink has less than one file in the queue, if true
    // and infinite flag is set reappend the queue of files in the sink
    // if false do nothing
    let rmp = Arc::clone(&rmplayer);
    let sink = rmp.get_sink();
    let sink = Arc::clone(sink);
    let infinite = rmp.get_infinte();
    let _sink_handler = thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_secs(1));

        // println!("sink len: {}", sink.lock().unwrap().len());
        let sink_len = { sink.lock().unwrap().len() };

        if sink_len <= 1 && infinite {
            rmp.fill_sink();
        }
    });

    let rmp = Arc::clone(&rmplayer);
    let input_handler = thread::spawn(move || {
        loop {
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => {
                        rmp.play_pause();
                    }
                    KeyCode::Char('s') => {
                        // shuffle
                    }
                    KeyCode::Char('n') | KeyCode::Char('l') => {
                        // next music in track list
                        rmp.next();
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
}
