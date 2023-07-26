use clap::Parser;
use rdev::{listen, Event};
use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time::Duration;

fn callback(event: Event) {
    match event.name {
        Some(string) => println!("User wrote {:?}", string),
        None => (),
    }
}

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

    thread::spawn(|| {
        // This will block.
        if let Err(error) = listen(callback) {
            println!("Error: {:?}", error)
        }
    });

    let file = BufReader::new(File::open(filepath).unwrap());
    let source = Decoder::new(file).unwrap();

    sink.append(source);

    thread::sleep(Duration::from_secs(5));

    sink.set_speed(3.0);

    sink.sleep_until_end();
}
