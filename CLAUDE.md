# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a terminal-based metronome application written in Rust that provides both constant and progressive BPM functionality. The application features a TUI (Terminal User Interface) with real-time audio playback and keyboard controls.

## Common Commands

- **Build**: `cargo build`
- **Run**: `cargo run -- --start-bpm <value> [--end-bpm <value>] [--duration <seconds>] [--measures <number>]`
- **Build optimized**: `cargo build --release`
- **Run optimized**: `cargo run --release -- --start-bpm <value>`
- **Check code**: `cargo check`
- **Format code**: `cargo fmt`
- **Lint code**: `cargo clippy`

## Architecture

The application follows a modular architecture with clear separation of concerns:

### Core Components

- **`main.rs`**: Entry point that coordinates the application lifecycle, initializes audio system, and spawns UI and metronome threads
- **`args.rs`**: Command-line argument parsing using clap, validates that duration/measures are provided together
- **`ui/mod.rs`**: TUI implementation using ratatui and crossterm for terminal interface and keyboard input handling
- **`metronome/mod.rs`**: Core metronome logic with support for both constant and progressive BPM modes
- **`audio/mod.rs`**: Audio playback system using rodio with embedded OGG audio file

### Threading Model

The application uses a hybrid async/threaded approach:
- **Main thread**: Async runtime (tokio) for coordination
- **UI thread**: Async task handling terminal interface and keyboard input
- **Metronome thread**: Synchronous thread for precise audio timing

### State Management

Shared state is managed using Arc<Mutex<T>> and AtomicBool for thread-safe communication:
- `bpm_shared`: Current BPM value (mutable, shared between UI and metronome)
- `running`: Application lifecycle control
- `paused`: Pause/resume state

### Progressive Mode

When both `--duration` and `--measures` are provided, the metronome operates in progressive mode:
- Calculates total beats based on average BPM and duration
- Increments BPM every N measures (specified by `--measures`)
- Transitions smoothly from start BPM to end BPM over the specified duration

## Key Features

- **Keyboard Controls**: J/K for BPM adjustment, Space for pause/resume, Q to quit
- **Real-time UI**: Live BPM display with pause indicator
- **Progressive BPM**: Gradual tempo changes over time with measure-based increments
- **Audio System**: Embedded OGG audio file for consistent tick sound across platforms
- **Precise Timing**: Uses Instant-based timing for accurate beat intervals

## Dependencies

- `clap`: Command-line argument parsing
- `ratatui` + `crossterm`: Terminal UI framework
- `rodio`: Cross-platform audio playback
- `tokio`: Async runtime for task coordination