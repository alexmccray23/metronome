use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetronomeState {
    Running = 0,
    Paused = 1,
    Stopped = 2,
}

impl From<u8> for MetronomeState {
    fn from(value: u8) -> Self {
        match value {
            0 => MetronomeState::Running,
            1 => MetronomeState::Paused,
            2 => MetronomeState::Stopped,
            _ => MetronomeState::Stopped,
        }
    }
}

pub struct AtomicMetronomeState {
    state: AtomicU8,
}

impl AtomicMetronomeState {
    pub fn new(initial_state: MetronomeState) -> Self {
        Self {
            state: AtomicU8::new(initial_state as u8),
        }
    }

    pub fn load(&self, ordering: Ordering) -> MetronomeState {
        MetronomeState::from(self.state.load(ordering))
    }

    pub fn store(&self, state: MetronomeState, ordering: Ordering) {
        self.state.store(state as u8, ordering);
    }

    pub fn compare_exchange(
        &self,
        current: MetronomeState,
        new: MetronomeState,
        success: Ordering,
        failure: Ordering,
    ) -> Result<MetronomeState, MetronomeState> {
        match self.state.compare_exchange(current as u8, new as u8, success, failure) {
            Ok(value) => Ok(MetronomeState::from(value)),
            Err(value) => Err(MetronomeState::from(value)),
        }
    }
}