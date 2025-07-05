use std::time::{Duration, Instant};

const MAX_TAP_HISTORY: usize = 5;
const TAP_TIMEOUT_MS: u64 = 5000;
const MIN_BPM: f64 = 5.0;
const MAX_BPM: f64 = 300.0;

#[derive(Debug)]
pub struct TapTempo {
    tap_times: Vec<Instant>,
    last_calculated_bpm: Option<f64>,
    is_tapping: bool,
    tap_timeout: Duration,
}

impl TapTempo {
    pub fn new() -> Self {
        Self {
            tap_times: Vec::with_capacity(MAX_TAP_HISTORY),
            last_calculated_bpm: None,
            is_tapping: false,
            tap_timeout: Duration::from_millis(TAP_TIMEOUT_MS),
        }
    }

    pub fn tap(&mut self) -> Option<f64> {
        let now = Instant::now();
        
        if let Some(last_tap) = self.tap_times.last() {
            if now.duration_since(*last_tap) > self.tap_timeout {
                self.tap_times.clear();
                self.is_tapping = false;
            }
        }

        self.tap_times.push(now);
        self.is_tapping = true;

        if self.tap_times.len() > MAX_TAP_HISTORY {
            self.tap_times.remove(0);
        }

        if self.tap_times.len() < 2 {
            return None;
        }

        let bpm = self.calculate_bpm();
        self.last_calculated_bpm = bpm;
        bpm
    }

    fn calculate_bpm(&self) -> Option<f64> {
        if self.tap_times.len() < 2 {
            return None;
        }

        let intervals: Vec<Duration> = self.tap_times
            .windows(2)
            .map(|pair| pair[1].duration_since(pair[0]))
            .collect();

        let total_duration: Duration = intervals.iter().sum();
        #[allow(clippy::cast_precision_loss)]
        let avg_interval_ms = total_duration.as_millis() as f64 / intervals.len() as f64;

        let bpm = 60000.0 / avg_interval_ms;

        if (MIN_BPM..=MAX_BPM).contains(&bpm) {
            Some(bpm)
        } else {
            None
        }
    }

    pub fn is_tapping(&self) -> bool {
        if !self.is_tapping {
            return false;
        }

        if let Some(last_tap) = self.tap_times.last() {
            let elapsed = Instant::now().duration_since(*last_tap);
            if elapsed > self.tap_timeout {
                return false;
            }
        }

        true
    }

    pub fn get_tap_count(&self) -> usize {
        if self.is_tapping() {
            self.tap_times.len()
        } else {
            0
        }
    }
}

impl Default for TapTempo {
    fn default() -> Self {
        Self::new()
    }
}
