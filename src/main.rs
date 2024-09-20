use clap::{Arg, Command};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};

use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

fn play_tick(stream_handle: &OutputStreamHandle) {
    let sink = Sink::try_new(stream_handle).unwrap();

    // Load a sound file
    let file = BufReader::new(File::open("/home/alexm/rust/metronome/assets/audio.ogg").unwrap());

    // Decode the sound file
    let tick = Decoder::new(file).unwrap();

    // Add the sound to the sink and detach to play asynchronously
    sink.append(tick);
    sink.detach();
}

fn capture_arguments() -> (f64, f64, Option<f64>, Option<u32>) {
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
                .required(false)
        )
        .arg(
            Arg::new("measures")
                .short('m')
                .long("measures")
                .help("Number of beats per BPM increment. Should be a multiple of the meter, e.g., 4, 32, 64, etc.")
                .required(false)
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
        .map(|b| b.parse::<u32>().expect("Invalid number of measures"));

    if duration.is_some() && measures.is_none() {
        eprintln!("Error: --duration must be provided when --measures is specified.");
        std::process::exit(1);
    }
    if duration.is_none() && measures.is_some() {
        eprintln!("Error: --duration must be provided when --measures is specified.");
        std::process::exit(1);
    }

    println!(
        "Metronome started from {start_bpm:.2} BPM to {end_bpm:.2} BPM.\n\
        Press 'k' or 'j' to adjust the BPM by 1\n\
        Press 'q' to quit"
    );

    (start_bpm, end_bpm, duration, measures)
}

fn start_key_listener(bpm: &Arc<Mutex<f64>>) {
    // Clone the Arc to move into the input thread
    let bpm_clone = Arc::clone(bpm);

    // Enable raw mode to capture key presses without enter key
    enable_raw_mode().expect("Failed to enable raw mode");

    // Input thread to listen for key presses
    thread::spawn(move || {
        loop {
            // Read an event
            if let Ok(Event::Key(key_event)) = read() {
                let mut bpm = bpm_clone.lock().unwrap();
                match key_event.code {
                    KeyCode::Char('k') => {
                        *bpm += 1.0;
                        println!("{:.2} BPM\r", *bpm);
                        drop(bpm);
                    }
                    KeyCode::Char('j') => {
                        if *bpm > 1.0 {
                            *bpm -= 1.0;
                            println!("{:.2} BPM\r", *bpm);
                            drop(bpm);
                        }
                    }
                    KeyCode::Char('q') => {
                        println!("Exiting metronome.\r");
                        drop(bpm);
                        disable_raw_mode().expect("Failed to disable raw mode\r");
                        std::process::exit(0);
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
    duration: Option<f64>,
    measures: Option<u32>,
    stream_handle: &OutputStreamHandle,
    bpm: &Arc<Mutex<f64>>,
) {
    let mut next_beat = Instant::now();

    // Calculate total steps (beats)
    let total_steps = duration.map_or(u32::MAX, |dur| {
        // Calculate number of beats over the duration
        let average_bpm = (start_bpm + end_bpm) / 2.0_f64;
        (average_bpm * (dur / 60.0)).round() as u32
    });

    // Calculate BPM increment per step
    if let Some(measures) = measures {
        let bpm_increment = if total_steps > 0 {
            f64::from(measures) * (end_bpm - start_bpm) / f64::from(total_steps)
        } else {
            0.0
        };

        let mut current_bpm = start_bpm;

        for beat in 0..total_steps {
            // Play the metronome tick
            play_tick(stream_handle);
            if beat == 0 {
                println!("{current_bpm:.2} BPM\r");
            }
            start_key_listener(bpm);

            // Calculate duration between beats
            let beat_duration = 60.0 / current_bpm;

            // Schedule the next beat
            next_beat += Duration::from_secs_f64(beat_duration);
            let now = Instant::now();

            if next_beat > now {
                sleep(next_beat - now);
            } else {
                next_beat = now;
            }

            // Update BPM
            if beat % measures == measures - 1 {
                current_bpm += bpm_increment;
                println!("{current_bpm:.2} BPM\r");
            }
        }
    }
    println!("{end_bpm:.2} BPM\r");
}

fn run_constant_metronome(bpm: &Arc<Mutex<f64>>, stream_handle: &OutputStreamHandle) {
    let mut next_beat = Instant::now();

    // Continue at end BPM if desired
    loop {
        // Get the current BPM
        let current_bpm = {
            let bpm = bpm.lock().unwrap();
            *bpm
        };
        start_key_listener(bpm);

        play_tick(stream_handle);

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

fn main() {
    // Capture command line arguments
    let (start_bpm, end_bpm, duration, measures) = capture_arguments();

    // Get an output stream handle to the default physical sound device
    if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
        let bpm = &mut Arc::new(Mutex::new(end_bpm));

        run_progressive_metronome(start_bpm, end_bpm, duration, measures, &stream_handle, bpm);

        run_constant_metronome(bpm, &stream_handle);
    }
}
