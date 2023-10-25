use clap::Parser;
use crossterm::event::{read, Event, KeyCode};
use crossterm::style::{Print, ResetColor};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{cursor, execute, QueueableCommand};
use rmp::RMPlayer;
use rodio::OutputStream;
use std::fs::metadata;
use std::io::{stdout, Write};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[derive(Parser)]
struct Args {
    path: Option<String>,
    #[arg(short = 's', long = "shuffle")]
    shuffle: bool,
    #[arg(short = 'i', long = "infinite")]
    infinite: bool,
}

fn print_current_song(rmp: &RMPlayer) {
    let current_playing = rmp.get_current_filename();
    println!("{}", current_playing.to_str().unwrap());

    let mut stdout = stdout();
    stdout.queue(cursor::MoveTo(0, 0)).unwrap();
    stdout.queue(Clear(ClearType::CurrentLine)).unwrap();
    stdout
        .queue(Print(current_playing.to_str().unwrap()))
        .unwrap();
    stdout.queue(ResetColor).unwrap();

    stdout.flush().unwrap();
}

fn print_volume(rmp: &RMPlayer) {
    let volume = { rmp.get_volume() };
    let volume = (volume * 10.0).round() / 10.0;

    let mut stdout = stdout();
    stdout.queue(cursor::MoveTo(0, 2)).unwrap();
    stdout.queue(Clear(ClearType::CurrentLine)).unwrap();
    stdout.queue(Print(volume.to_string())).unwrap();
    stdout.queue(ResetColor).unwrap();

    stdout.flush().unwrap();
}

enum ChannelCommands {
    PlayPause,
    Next,
    Previous,
    VolumeUp,
    VolumeDown,
    Forward,
    Backward,
    Shuffle,
}

fn check_fill_sink(rmp: &RMPlayer) {
    if rmp.get_sink_len() <= 1 && rmp.get_infinte() {
        rmp.fill_sink();
    }
}

fn main() {
    let args = Args::parse();
    let filepath = args.path.unwrap();

    let filepath = Path::new(&filepath);

    let md = metadata(filepath).unwrap();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let rmplayer = RMPlayer::new(
        stream_handle.clone(),
        filepath.to_str().unwrap().to_string(),
        args.shuffle,
        args.infinite,
    );
    let (tx, rx): (Sender<ChannelCommands>, Receiver<ChannelCommands>) = mpsc::channel();

    assert!(
        !md.is_file(),
        "Not implemented -> start with file and continue with music saved in db"
    );

    rmplayer.fill_sink();

    enable_raw_mode().unwrap();

    // clear terminal
    execute!(stdout(), Clear(ClearType::All)).unwrap();

    // print to screen current music
    print_current_song(&rmplayer);

    // checks every 1 second if the sink has less than one file in the queue, if true
    // and infinite flag is set reappend the queue of files in the sink
    // if false do nothing
    let _sink_handler = thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_millis(100));

        // check for user input
        if let Ok(input) = rx.try_recv() {
            match input {
                ChannelCommands::PlayPause => rmplayer.play_pause(),
                ChannelCommands::Next => {
                    check_fill_sink(&rmplayer);
                    rmplayer.next();
                }
                ChannelCommands::Previous => panic!("TODO: go to previous track"),
                ChannelCommands::VolumeUp => rmplayer.volume_up(),
                ChannelCommands::VolumeDown => rmplayer.volume_down(),
                ChannelCommands::Forward => panic!("TODO: forward 5 seconds"),
                ChannelCommands::Backward => panic!("TODO: backward 5 seconds"),
                ChannelCommands::Shuffle => panic!("TODO: shuffle"),
            }
        }

        // print current song title on the terminal
        print_current_song(&rmplayer);

        // append tracks to the sink again if only one remain
        check_fill_sink(&rmplayer);
    });

    let input_handler = thread::spawn(move || {
        loop {
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => {
                        tx.send(ChannelCommands::PlayPause).unwrap();
                    }
                    KeyCode::Char('s') => {
                        // shuffle
                        tx.send(ChannelCommands::Shuffle).unwrap();
                    }
                    KeyCode::Char('n') | KeyCode::Char('l') => {
                        // next music in track list
                        tx.send(ChannelCommands::Next).unwrap();
                    }
                    KeyCode::Char('p') | KeyCode::Char('h') => {
                        // previous in track list
                        tx.send(ChannelCommands::Previous).unwrap();
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        tx.send(ChannelCommands::VolumeUp).unwrap();
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        // volume down
                        tx.send(ChannelCommands::VolumeDown).unwrap();
                    }
                    KeyCode::Right => {
                        // volume go forward
                        tx.send(ChannelCommands::Forward).unwrap();
                    }
                    KeyCode::Left => {
                        // volume go backward
                        tx.send(ChannelCommands::Backward).unwrap();
                    }
                    _ => {}
                }
            }
        }
    });

    input_handler.join().unwrap();

    disable_raw_mode().unwrap();
}
