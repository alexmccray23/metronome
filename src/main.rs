mod args;
mod audio;
mod metronome;
mod ui;

use std::sync::{atomic::AtomicBool, Arc, Mutex};
use tokio::task::JoinHandle;
use rodio::OutputStreamHandle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let (start_bpm, end_bpm, duration_opt, measures_opt) = args::parse_arguments();

    // Initialize audio system
    if let Ok((_stream, stream_handle)) = rodio::OutputStream::try_default() {
        // Shared state
        let bpm_shared = Arc::new(Mutex::new(start_bpm));
        let running = Arc::new(AtomicBool::new(true));
        let paused = Arc::new(AtomicBool::new(false));

        // Start UI and metronome
        let ui_handle = start_ui(&bpm_shared, &running, &paused, start_bpm);
        start_metronome(
            &stream_handle,
            &bpm_shared,
            &running,
            &paused,
            start_bpm,
            end_bpm,
            duration_opt,
            measures_opt,
        );

        // Wait for UI to complete
        let _ = tokio::join!(ui_handle);
    } else {
        eprintln!("Error: Unable to access audio output stream.");
    }

    Ok(())
}

fn start_ui(
    bpm_shared: &Arc<Mutex<f64>>,
    running: &Arc<AtomicBool>,
    paused: &Arc<AtomicBool>,
    start_bpm: f64,
) -> JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> {
    tokio::spawn(ui::run(
        Arc::clone(bpm_shared),
        Arc::clone(running),
        Arc::clone(paused),
        start_bpm,
    ))
}

fn start_metronome(
    stream_handle: &OutputStreamHandle,
    bpm_shared: &Arc<Mutex<f64>>,
    running: &Arc<AtomicBool>,
    paused: &Arc<AtomicBool>,
    start_bpm: f64,
    end_bpm: f64,
    duration_opt: Option<f64>,
    measures_opt: Option<u32>,
) {
    std::thread::spawn(move || {
        if let (Some(duration), Some(measures)) = (duration_opt, measures_opt) {
            let args = metronome::ProgressiveArgs::new(start_bpm, end_bpm, duration, measures);
            metronome::run_progressive(&args, stream_handle, bpm_shared, running, paused);
        }
        metronome::run_constant(bpm_shared, stream_handle, running, paused);
    });
}
