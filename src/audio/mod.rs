use rodio::{Decoder, OutputStreamHandle, Sink};
use std::io::{BufReader, Cursor};
use std::sync::atomic::{AtomicBool, Ordering};

pub fn play_tick(stream_handle: &OutputStreamHandle, paused: &AtomicBool) {
    let sink = Sink::try_new(stream_handle).unwrap();

    if paused.load(Ordering::SeqCst) {
        sink.pause();
    } else {
        sink.play();
    }

    let audio_data = include_bytes!("../../assets/audio.ogg");
    let cursor = Cursor::new(&audio_data[..]);
    let tick = Decoder::new(BufReader::new(cursor)).unwrap();

    sink.append(tick);
    sink.detach();
}
