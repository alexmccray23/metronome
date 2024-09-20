use clap::{Arg, Command};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::{BufReader, Cursor};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};

use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

fn play_tick(stream_handle: &OutputStreamHandle) {
    let sink = Sink::try_new(stream_handle).unwrap();

    // Embed the audio file into the binary
    let audio_data = include_bytes!("../assets/audio.ogg");
    let cursor = Cursor::new(&audio_data[..]);

    // Decode the audio data
    let tick = Decoder::new(BufReader::new(cursor)).unwrap();

    // Add the sound to the sink and detach to play asynchronously
    sink.append(tick);
    sink.detach();
}

fn parse_arguments() -> (f64, f64, Option<f64>, Option<u32>) {
    // Set up command-line argument parsing
    let matches = Command::new("Metronome")
        .version("1.0")
        .about("A simple metronome that can progressively speed up")
        .arg(
            Arg::new("start-bpm")
                .short('s')
                .long("start-bpm")
                .help("Starting BPM")
                .required(true),
        )
        .arg(
            Arg::new("end-bpm")
                .short('e')
                .long("end-bpm")
                .help("Ending BPM")
                .required(false),
        )
        .arg(
            Arg::new("duration")
                .short('d')
                .long("duration")
                .help("Duration over which BPM changes (in seconds)")
                .required(false),
        )
        .arg(
            Arg::new("measures")
                .short('m')
                .long("measures")
                .help("Number of beats per BPM increment. Should be a multiple of the meter, e.g., 4, 32, 64, etc.")
                .required(false),
        )
        .get_matches();

    // Parse BPM values
    let start_bpm = matches
        .get_one::<String>("start-bpm")
        .expect("Invalid starting BPM")
        .parse::<f64>()
        .expect("Invalid starting BPM");

    let end_bpm = matches
        .get_one::<String>("end-bpm")
        .unwrap_or(&start_bpm.to_string())
        .parse::<f64>()
        .expect("Invalid ending BPM");

    let duration = matches
        .get_one::<String>("duration")
        .map(|d| d.parse::<f64>().expect("Invalid duration"));

    let measures = matches
        .get_one::<String>("measures")
        .map(|m| m.parse::<u32>().expect("Invalid number of measures"));

    // Validate arguments
    if duration.is_some() && measures.is_none() || duration.is_none() && measures.is_some() {
        eprintln!("Error: Both --duration and --measures must be provided together.");
        std::process::exit(1);
    }

    println!(
        "Metronome started from {start_bpm:.2} BPM to {end_bpm:.2} BPM.\n\
        Press 'k' or '+' to increase the BPM by 1\n\
        Press 'j' or '-' to decrease the BPM by 1\n\
        Press 'q' to quit\r"
    );

    (start_bpm, end_bpm, duration, measures)
}

fn start_key_listener(bpm: Arc<Mutex<f64>>, running: Arc<AtomicBool>) {
    // Enable raw mode to capture key presses without enter key
    enable_raw_mode().expect("Failed to enable raw mode");

    // Input thread to listen for key presses
    thread::spawn(move || {
        while running.load(Ordering::SeqCst) {
            // Read an event
            if let Ok(Event::Key(key_event)) = read() {
                match key_event.code {
                    KeyCode::Char('k' | '+') => {
                        {
                            let mut bpm = bpm.lock().unwrap();
                            *bpm += 1.0;
                            println!("{:.2} BPM\r", *bpm);
                            drop(bpm);
                        } // MutexGuard dropped here
                    }
                    KeyCode::Char('j' | '-') => {
                        {
                            let mut bpm = bpm.lock().unwrap();
                            if *bpm > 1.0 {
                                *bpm -= 1.0;
                                println!("{:.2} BPM\r", *bpm);
                                drop(bpm);
                            }
                        } // MutexGuard dropped here
                    }
                    KeyCode::Char('q') => {
                        println!("Exiting metronome.\r");
                        running.store(false, Ordering::SeqCst);
                        disable_raw_mode().expect("Failed to disable raw mode");
                        return;
                    }
                    _ => {}
                }
            }
        }
    });
}

fn run_progressive_metronome(
    start_bpm: f64,
    end_bpm: f64,
    duration: f64,
    measures: u32,
    stream_handle: &OutputStreamHandle,
    bpm_shared: &Arc<Mutex<f64>>,
    running: &Arc<AtomicBool>,
) {
    // Calculate total beats over the duration
    let average_bpm = (start_bpm + end_bpm) / 2.0;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let total_beats = (average_bpm * (duration / 60.0)).round() as u32;

    // Calculate the number of increments and BPM increment size
    let num_increments = total_beats / measures;
    let bpm_increment = if num_increments > 0 {
        (end_bpm - start_bpm) / f64::from(num_increments)
    } else {
        0.0
    };

    let mut current_bpm = start_bpm;
    let mut next_beat = Instant::now();

    for beat in 0..total_beats {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        // Play the metronome tick
        play_tick(stream_handle);
        if beat == 0 {
            println!("{current_bpm:.2} BPM\r");
        }

        // Calculate duration between beats
        let beat_duration = 60.0 / current_bpm;

        // Schedule the next beat
        next_beat += Duration::from_secs_f64(beat_duration);
        let now = Instant::now();

        if next_beat > now {
            sleep(next_beat - now);
        } else {
            // We're behind schedule
            next_beat = now;
        }

        // Update BPM after each increment
        if (beat + 1) % measures == 0 && (beat + 1) < total_beats {
            current_bpm += bpm_increment;
            println!("{current_bpm:.2} BPM\r");
            // Update the shared BPM
            {
                let mut bpm = bpm_shared.lock().unwrap();
                *bpm = current_bpm;
            }
        }
    }

    // Ensure shared BPM is set to end_bpm
    {
        let mut bpm = bpm_shared.lock().unwrap();
        *bpm = end_bpm;
    }
    println!("{end_bpm:.2} BPM\r");
}

fn run_constant_metronome(
    bpm_shared: &Arc<Mutex<f64>>,
    stream_handle: &OutputStreamHandle,
    running: &Arc<AtomicBool>,
) {
    let mut next_beat = Instant::now();

    while running.load(Ordering::SeqCst) {
        // Get the current BPM
        let current_bpm = {
            let bpm = bpm_shared.lock().unwrap();
            *bpm
        };

        play_tick(stream_handle);

        let beat_duration = 60.0 / current_bpm;

        next_beat += Duration::from_secs_f64(beat_duration);

        let now = Instant::now();

        if next_beat > now {
            sleep(next_beat - now);
        } else {
            // We're behind schedule
            next_beat = now;
        }
    }
}

fn main() {
    // Capture command line arguments
    let (start_bpm, end_bpm, duration_opt, measures_opt) = parse_arguments();

    // Get an output stream handle to the default physical sound device
    if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
        // Shared BPM variable
        let bpm_shared = Arc::new(Mutex::new(end_bpm));
        // Shared running flag for graceful shutdown
        let running = Arc::new(AtomicBool::new(true));

        // Start the key listener thread
        start_key_listener(Arc::clone(&bpm_shared), Arc::clone(&running));

        // Run progressive metronome if duration and measures are provided
        if let (Some(duration), Some(measures)) = (duration_opt, measures_opt) {
            run_progressive_metronome(
                start_bpm,
                end_bpm,
                duration,
                measures,
                &stream_handle,
                &bpm_shared,
                &running,
            );
        }

        // Run constant metronome at the end BPM
        run_constant_metronome(&bpm_shared, &stream_handle, &running);

        // Disable raw mode before exiting
        disable_raw_mode().expect("Failed to disable raw mode");
    } else {
        eprintln!("Error: Unable to access audio output stream.");
    }
}
