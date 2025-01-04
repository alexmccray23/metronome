use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};
use rodio::OutputStreamHandle;

pub struct ProgressiveArgs {
    pub start_bpm: f64,
    pub end_bpm: f64,
    pub duration: f64,
    pub measures: u32,
}

impl ProgressiveArgs {
    pub const fn new(start_bpm: f64, end_bpm: f64, duration: f64, measures: u32) -> Self {
        Self {
            start_bpm,
            end_bpm,
            duration,
            measures,
        }
    }
}

pub fn run_progressive(
    args: &ProgressiveArgs,
    stream_handle: &OutputStreamHandle,
    bpm_shared: &Arc<Mutex<f64>>,
    running: &AtomicBool,
    paused: &AtomicBool,
) {
    let average_bpm = (args.start_bpm + args.end_bpm) / 2.0;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let total_beats = (average_bpm * (args.duration / 60.0)).round() as u32;

    let num_increments = total_beats / args.measures;
    let bpm_increment = if num_increments > 0 {
        (args.end_bpm - args.start_bpm) / f64::from(num_increments)
    } else {
        0.0
    };

    let mut current_bpm = args.start_bpm;
    let mut next_beat = Instant::now();

    for beat in 0..total_beats {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        super::audio::play_tick(stream_handle, paused);

        while paused.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(100));
            if !running.load(Ordering::SeqCst) {
                return;
            }
        }

        let beat_duration = 60.0 / current_bpm;
        next_beat += Duration::from_secs_f64(beat_duration);
        let now = Instant::now();

        if next_beat > now {
            sleep(next_beat - now);
        } else {
            next_beat = now;
        }

        if (beat + 1) % args.measures == 0 && (beat + 1) < total_beats {
            current_bpm += bpm_increment;
            {
                let mut bpm = bpm_shared.lock().unwrap();
                *bpm = current_bpm;
            }
        }
    }

    {
        let mut bpm = bpm_shared.lock().unwrap();
        *bpm = args.end_bpm;
    }
}

pub fn run_constant(
    bpm_shared: &Arc<Mutex<f64>>,
    stream_handle: &OutputStreamHandle,
    running: &AtomicBool,
    paused: &AtomicBool,
) {
    let mut next_beat = Instant::now();

    while running.load(Ordering::SeqCst) {
        let current_bpm = {
            let bpm = bpm_shared.lock().unwrap();
            *bpm
        };

        super::audio::play_tick(stream_handle, paused);

        let beat_duration = 60.0 / current_bpm;
        next_beat += Duration::from_secs_f64(beat_duration);

        let now = Instant::now();
        if next_beat > now {
            sleep(next_beat - now);
        } else {
            next_beat = now;
        }
    }
}
