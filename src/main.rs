use clap::Parser;
use rdev::listen;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{mpsc, Arc, Mutex};
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

    // thread::sleep(Duration::from_secs(5));

    let (sender, receiver) = mpsc::channel();
    // let receiver = Arc::new(Mutex::new(receiver));

    // listener thread
    let _listener = thread::spawn(move || {
        listen(move |event| {
            sender
                .send(event)
                .unwrap_or_else(|e| println!("Could not send event {:?}", e));
        })
    });

    println!("Listening for events");
    let receiver = thread::spawn(move || {
        for event in receiver.iter() {
            println!("Event: {:?}", event);
            if let Some(input) = event.name {
                let input = input.to_lowercase();

                match input.as_str() {
                    " " => {
                        if sink.is_paused() {
                            sink.play();
                        } else {
                            sink.pause();
                        }
                    }
                    "q" => {
                        thread::sleep(std::time::Duration::from_millis(50));
                        sink.stop();
                        return;
                    }
                    "up" => {
                        sink.set_volume(sink.volume() + 0.1);
                    }
                    "down" => {
                        sink.set_volume(sink.volume() - 0.1);
                    }
                    _ => {}
                }
            }
        }
    });

    receiver.join().unwrap();
}
