use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MetronomeState {
    Running,
    Paused,
    Stopped,
}

impl From<u8> for MetronomeState {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Running,
            1 => Self::Paused,
            _ => Self::Stopped,
        }
    }
}

pub struct AtomicMetronomeState {
    state: AtomicU8,
}

impl AtomicMetronomeState {
    pub const fn new(initial_state: MetronomeState) -> Self {
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
}
