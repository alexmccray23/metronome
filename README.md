# Metronome

A terminal-based metronome application built in Rust with a clean TUI interface. Features both constant and progressive tempo modes with real-time BPM adjustment and tap tempo functionality.

## Features

- **Constant Tempo**: Maintains steady BPM with real-time adjustment
- **Progressive Tempo**: Gradually increases BPM over a specified duration
- **Tap Tempo**: Calculate BPM by tapping a key
- **Interactive TUI**: Clean terminal interface with keyboard controls
- **Real-time Control**: Adjust BPM on-the-fly without stopping playback

## Installation

### Prerequisites

- Rust 1.70+ (with Cargo)
- Audio output capability

### Building from Source

```bash
git clone https://github.com/alexmccray23/metronome.git
cd metronome
cargo build --release
```

The executable will be available at `target/release/metronome`.

## Usage

### Basic Usage

Start a constant tempo metronome:

```bash
metronome --start-bpm 120
```

### Progressive Tempo

Gradually increase from 60 to 120 BPM over 5 minutes, changing every 32 beats:

```bash
metronome --start-bpm 60 --end-bpm 120 --duration 300 --measures 32
```

### Command Line Options

- `--start-bpm, -s`: Starting BPM (required)
- `--end-bpm, -e`: Ending BPM (optional, defaults to start-bpm)
- `--duration, -d`: Duration in seconds for tempo change (requires --measures)
- `--measures, -m`: Number of beats between BPM increments (requires --duration)

## Controls

Once running, use these keyboard controls:

- **J/j**: Decrease BPM by 1
- **K/k**: Increase BPM by 1
- **Space**: Pause/Resume
- **G/g**: Tap tempo (tap multiple times to set BPM)
- **I/i** or **Enter**: Manual BPM input mode
- **Q/q**: Quit

### Manual Input Mode

Press `I` or `Enter` to enter manual BPM input mode:
- Type a number (decimals supported)
- Press `Enter` to confirm
- Press `Esc` to cancel

## Examples

### Practice Session
```bash
# Start slow and gradually speed up
metronome --start-bpm 80 --end-bpm 140 --duration 600 --measures 16
```

### Metronome for 4/4 Time
```bash
# Use multiples of 4 for measures parameter
metronome --start-bpm 100 --end-bpm 120 --duration 240 --measures 4
```

### Simple Constant Tempo
```bash
metronome --start-bpm 120
```

## Technical Details

- Built with Rust for cross-platform compatibility
- Uses Ratatui for the terminal user interface
- Audio playback via Rodio
- Multi-threaded architecture for responsive UI and precise timing
- Thread-safe state management with atomic operations

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
