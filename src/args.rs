use clap::{Arg, Command};

pub fn parse_arguments() -> (f64, f64, Option<f64>, Option<u32>) {
    let matches = Command::new("Metronome")
        .version("1.0")
        .about("A simple TUI metronome that can progressively speed up")
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

    if duration.is_some() && measures.is_none() || duration.is_none() && measures.is_some() {
        eprintln!("Error: Both --duration and --measures must be provided together.");
        std::process::exit(1);
    }

    (start_bpm, end_bpm, duration, measures)
}
