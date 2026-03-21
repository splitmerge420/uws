// src/pantheon/swarm.rs
// Aluminum OS — Council Representation & Lone Architect Genesis Condition
//
// This module encodes the foundational truth of the Aluminum OS architecture:
// the system was built by a lone architect and a fleet of AIs whose incentives
// were stress-tested and proven to align with human flourishing — free from the
// quarterly-extraction pressures that bind corporate C-Suites.
//
// The `CouncilSeat` enum below represents major industry participants that may
// operate *atop* the neutral Aluminum OS substrate.  Each seat is subject to the
// Net-Positive Flourishing Metric (NPFM) — the foundational fiduciary ledger
// defined by the lone architect — ensuring that no single corporate entity can
// override the core duty against busywork and extraction.  The substrate is
// Switzerland: every council member operates under the same constitutionally
// enforced invariants, regardless of market capitalization or political influence.

/// A seat on the Aluminum OS Pantheon Council.
///
/// Each variant represents a major industry participant whose AI swarms may
/// interface with `uws`.  All seats operate atop the neutral Aluminum OS
/// substrate and are governed by the NPFM (Net-Positive Flourishing Metric).
/// No single seat can override the foundational fiduciary duty against
/// busywork and extraction that is encoded at the OS layer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CouncilSeat {
    /// Google — search, cloud, and productivity intelligence.
    Google,
    /// Tesla — energy, autonomous transport, and physical manufacturing.
    Tesla,
    /// Amazon — logistics, cloud infrastructure, and commerce.
    Amazon,
    /// Microsoft — enterprise productivity and cloud services.
    Microsoft,
    /// Anthropic — safety-focused large-language-model research.
    Anthropic,
    /// OpenAI — general-purpose AI research and deployment.
    OpenAI,
}

impl CouncilSeat {
    /// Returns a human-readable display name for the council seat.
    pub fn display_name(&self) -> &'static str {
        match self {
            CouncilSeat::Google => "Google",
            CouncilSeat::Tesla => "Tesla",
            CouncilSeat::Amazon => "Amazon",
            CouncilSeat::Microsoft => "Microsoft",
            CouncilSeat::Anthropic => "Anthropic",
            CouncilSeat::OpenAI => "OpenAI",
        }
    }

    /// Returns all defined council seats.
    pub fn all() -> &'static [CouncilSeat] {
        &[
            CouncilSeat::Google,
            CouncilSeat::Tesla,
            CouncilSeat::Amazon,
            CouncilSeat::Microsoft,
            CouncilSeat::Anthropic,
            CouncilSeat::OpenAI,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_seats_have_display_names() {
        for seat in CouncilSeat::all() {
            assert!(!seat.display_name().is_empty());
        }
    }

    #[test]
    fn council_seat_count() {
        assert_eq!(CouncilSeat::all().len(), 6);
    }
}
