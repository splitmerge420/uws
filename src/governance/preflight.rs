/// Preflight classification states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreFlightState {
    Allowed,
    Review,
    Restricted,
    Blocked,
}

/// Result of a preflight evaluation.
#[derive(Debug, Clone)]
pub struct PreFlightResult {
    pub state: PreFlightState,
    pub should_block: bool,
}

/// Skeleton four-state classifier gate.
pub struct FourStateClassifierGate;

impl FourStateClassifierGate {
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluate an input query or manifest.
    ///
    /// Phase 1: always returns Allowed.
    /// Phase 1.5: replace with House 12 + INV-aware classification.
    pub fn evaluate(&self, _input: &str) -> PreFlightResult {
        PreFlightResult {
            state: PreFlightState::Allowed,
            should_block: false,
        }
    }
}
