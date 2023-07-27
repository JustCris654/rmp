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

    let events_queue = Arc::new(Mutex::new(Vec::new()));
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
    let events_queue_recv = Arc::clone(&events_queue);
    let receiver = thread::spawn(move || {
        for event in receiver.iter() {
            println!("Event: {:?}", event);
            if let Some(input) = event.name {
                let input = input.to_lowercase();
                events_queue_recv.lock().unwrap().push(input);
            }
        }
    });

    sink.set_volume(0.5);

    println!("Handling events");
    let events_queue_send = Arc::clone(&events_queue);
    let handler = thread::spawn(move || loop {
        if let Some(event) = events_queue_send.lock().unwrap().pop() {
            match event.as_str() {
                " " => {
                    if sink.is_paused() {
                        sink.play();
                    } else {
                        sink.pause();
                    }
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
    });

    handler.join().unwrap();
    receiver.join().unwrap();
}
