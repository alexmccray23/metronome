mod args;
mod audio;
mod metronome;
mod state;
mod tap_tempo;
mod ui;

use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use rodio::OutputStreamHandle;
use state::{AtomicMetronomeState, MetronomeState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let (start_bpm, end_bpm, duration_opt, measures_opt) = args::parse_arguments();

    // Initialize audio system
    if let Ok((_stream, stream_handle)) = rodio::OutputStream::try_default() {
        // Shared state
        let bpm_shared = Arc::new(Mutex::new(start_bpm));
        let state = Arc::new(AtomicMetronomeState::new(MetronomeState::Running));

        // Start UI and metronome
        let ui_handle = start_ui(&bpm_shared, &state, start_bpm);
        start_metronome(
            stream_handle,
            bpm_shared,
            state,
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
    state: &Arc<AtomicMetronomeState>,
    start_bpm: f64,
) -> JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> {
    tokio::spawn(ui::run(
        Arc::clone(bpm_shared),
        Arc::clone(state),
        start_bpm,
    ))
}

fn start_metronome(
    stream_handle: OutputStreamHandle,
    bpm_shared: Arc<Mutex<f64>>,
    state: Arc<AtomicMetronomeState>,
    start_bpm: f64,
    end_bpm: f64,
    duration_opt: Option<f64>,
    measures_opt: Option<u32>,
) {
    std::thread::spawn(move || {
        if let (Some(duration), Some(measures)) = (duration_opt, measures_opt) {
            let args = metronome::ProgressiveArgs::new(start_bpm, end_bpm, duration, measures);
            metronome::run_progressive(&args, &stream_handle, &bpm_shared, &state);
        }
        metronome::run_constant(&bpm_shared, &stream_handle, &state);
    });
}
