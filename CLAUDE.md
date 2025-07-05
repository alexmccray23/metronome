# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust terminal-based metronome application with TUI (Terminal User Interface) using Ratatui. It supports both constant and progressive tempo modes, where BPM can gradually increase over time.

## Build and Development Commands

```bash
# Build the project
cargo build

# Run in development mode
cargo run -- --start-bpm 120

# Build optimized release
cargo build --release

# Run with progressive tempo example
cargo run -- --start-bpm 60 --end-bpm 120 --duration 300 --measures 32

# Check code
cargo check

# Run tests
cargo test

# Format code
cargo fmt

# Run clippy linter
cargo clippy
```

## Architecture

The application follows a multi-threaded architecture with shared state management:

- **main.rs**: Entry point that coordinates UI and metronome threads
- **metronome.rs**: Core metronome logic with two modes:
  - `run_constant()`: Maintains steady BPM
  - `run_progressive()`: Gradually changes BPM over time
- **state.rs**: Thread-safe atomic state management (`AtomicMetronomeState`)
- **ui.rs**: TUI using Ratatui with keyboard controls and real-time display
- **audio.rs**: Audio playback using Rodio with embedded OGG file
- **tap_tempo.rs**: Tap tempo calculation with rolling average
- **args.rs**: Command-line argument parsing with Clap

## Key Components

### State Management
- `AtomicMetronomeState`: Thread-safe enum wrapper (Running/Paused/Stopped)
- `Arc<Mutex<f64>>`: Shared BPM value across threads
- Real-time state synchronization between UI and audio threads

### UI Controls
- `j/J`: Decrease BPM by 1
- `k/K`: Increase BPM by 1  
- `Space`: Pause/Resume
- `g/G`: Tap tempo
- `i/I` or `Enter`: Manual BPM input mode
- `q/Q`: Quit

### Progressive Mode
Requires both `--duration` and `--measures` parameters. BPM changes are calculated based on total beats and distributed across measure boundaries.

## Threading Model

- **Main thread**: Coordinates startup and shutdown
- **UI thread**: Tokio async task handling user input and display
- **Audio thread**: Native thread for precise timing and audio playback

## Audio System

Uses Rodio for cross-platform audio with an embedded OGG file (`assets/audio.ogg`). Each tick creates a new `Sink` that detaches after playing.

## Dependencies

- `ratatui`: TUI framework
- `crossterm`: Terminal control
- `rodio`: Audio playback
- `clap`: CLI argument parsing
- `tokio`: Async runtime for UI