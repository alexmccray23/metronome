use clap::{Arg, Command};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::{BufReader, Cursor};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::sleep;
use std::time::{Duration, Instant};

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

#[derive(Debug)]
struct AppState {
    current_bpm: f64,
    is_running: bool,
    is_paused: bool,
}

async fn run_ui(
    bpm_shared: Arc<Mutex<f64>>,
    running: Arc<AtomicBool>,
    start_bpm: f64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = AppState {
        current_bpm: start_bpm,
        is_running: true,
        is_paused: false,
    };

    while app_state.is_running {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(f.area());

            let bpm_text = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        format!("{:.2}", app_state.current_bpm),
                        Style::default().fg(Color::Green),
                    ),
                    Span::raw(" BPM  "),
                ]),
            ];

            let bpm_block = Paragraph::new(bpm_text).centered().block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(" Metronome ".blue().bold()).centered()),
            );
            f.render_widget(bpm_block, chunks[0]);

            let controls_text = vec![Line::from(vec![
                "Decrease BPM: ".into(),
                "<J>".blue(),
                " Increase BPM: ".into(),
                "<K>".blue(),
                " Pause/Resume: ".into(),
                "<Space>".blue(),
                " Quit: ".into(),
                "<Q>".blue(),
            ])
            .centered()];

            let controls_block = Paragraph::new(controls_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(" Controls ".yellow().bold()).centered()),
            );
            f.render_widget(controls_block, chunks[1]);
        })?;

        if let Ok(new_bpm) = bpm_shared.lock() {
            app_state.current_bpm = *new_bpm;
        }

        // Check for key events with a shorter timeout
        if event::poll(Duration::from_millis(16))? {
            // ~60Hz polling
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('k') => {
                        let mut bpm = bpm_shared.lock().unwrap();
                        *bpm += 1.0;
                        app_state.current_bpm = *bpm;
                    }
                    KeyCode::Char('j') => {
                        let mut bpm = bpm_shared.lock().unwrap();
                        if *bpm > 1.0 {
                            *bpm -= 1.0;
                            app_state.current_bpm = *bpm;
                        }
                    }
                    KeyCode::Char('q') => {
                        app_state.is_running = false;
                        running.store(false, Ordering::SeqCst);
                    }
                    KeyCode::Char(' ') => {
                        app_state.is_paused = !app_state.is_paused;
                        // Update the paused state in the UI
                        let paused_text = if app_state.is_paused {
                            " [PAUSED]".red()
                        } else {
                            "".into()
                        };
                        let bpm_text = vec![
                            Line::from(""),
                            Line::from(vec![
                                Span::styled(
                                    format!("{:.2}", app_state.current_bpm),
                                    Style::default().fg(Color::Green),
                                ),
                                Span::raw(" BPM  "),
                                paused_text,
                            ]),
                        ];
                        let bpm_block = Paragraph::new(bpm_text).centered().block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(Line::from(" Metronome ".blue().bold()).centered()),
                        );
                        terminal.draw(|f| {
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                                .split(f.area());
                            f.render_widget(bpm_block, chunks[0]);
                        })?;
                    }
                    _ => {}
                }
            }
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    Ok(())
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
        Press 'K' to increase the BPM by 1\n\
        Press 'J' to decrease the BPM by 1\n\
        Press 'Q' to quit\r"
    );

    (start_bpm, end_bpm, duration, measures)
}

fn run_progressive_metronome(
    start_bpm: f64,
    end_bpm: f64,
    duration: f64,
    measures: u32,
    stream_handle: &OutputStreamHandle,
    bpm_shared: &Arc<Mutex<f64>>,
    running: &Arc<AtomicBool>,
    paused: &Arc<AtomicBool>,
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

        // Skip if paused
        while paused.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(100));
            if !running.load(Ordering::SeqCst) {
                return;
            }
        }

        // Play the metronome tick
        play_tick(stream_handle);

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
}

fn run_constant_metronome(
    bpm_shared: &Arc<Mutex<f64>>,
    stream_handle: &OutputStreamHandle,
    running: &Arc<AtomicBool>,
    paused: &Arc<AtomicBool>,
) {
    let mut next_beat = Instant::now();

    while running.load(Ordering::SeqCst) {
        // Skip if paused
        while paused.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(100));
            if !running.load(Ordering::SeqCst) {
                return;
            }
        }

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Capture command line arguments
    let (start_bpm, end_bpm, duration_opt, measures_opt) = parse_arguments();

    // Get an output stream handle to the default physical sound device
    if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
        // Shared BPM variable
        let bpm_shared = Arc::new(Mutex::new(start_bpm));
        // Shared running flag for graceful shutdown
        let running = Arc::new(AtomicBool::new(true));
        let paused = Arc::new(AtomicBool::new(false));

        // Start UI in a separate task
        let ui_handle = tokio::spawn(run_ui(
            Arc::clone(&bpm_shared),
            Arc::clone(&running),
            start_bpm,
        ));

        // Start metronome in a separate thread
        let metronome_handle = std::thread::spawn(move || {
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
                    &paused,
                );
            }

            // Run constant metronome at the end BPM
            run_constant_metronome(&bpm_shared, &stream_handle, &running, &paused);
        });

        // Wait for both tasks to complete
        let _ = tokio::join!(ui_handle);
        metronome_handle.join().unwrap();
    } else {
        eprintln!("Error: Unable to access audio output stream.");
    }

    Ok(())
}
