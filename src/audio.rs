use rodio::{Decoder, OutputStreamHandle, Sink};
use std::io::{BufReader, Cursor};

pub fn play_tick(stream_handle: &OutputStreamHandle) {
    let sink = Sink::try_new(stream_handle).unwrap();

    let audio_data = include_bytes!("../assets/audio.ogg");
    let cursor = Cursor::new(&audio_data[..]);
    let tick = Decoder::new(BufReader::new(cursor)).unwrap();

    sink.append(tick);
    sink.detach();
}
