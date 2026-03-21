// src/pantheon/swarm.rs
// Aluminum OS — Pantheon Council Seats
//
// These council seats represent the major AI and technology organizations that
// operate atop the neutral Aluminum OS substrate. The substrate itself was
// originated by a lone systems architect (using a fleet of AIs and GitHub)
// whose incentives were stress-tested and proven to align purely with human
// flourishing — free from quarterly extraction pressures and corporate lock-in.
//
// By building the foundation outside any single megacorp, Aluminum OS functions
// as a neutral commons: a "Switzerland" of the AI stack. Each seat below has
// equal constitutional standing. No seat can override the NPFM (Net-Positive
// Flourishing Metric), and no single corporate entity can capture the
// foundational economic or moral ledger.
//
// Governance reference: ALUMINUM_OS_WHITEPAPER.md § 5.3 — "The Pantheon Council Model"
// Constitutional invariant: INV-7 (Vendor Balance), INV-5 (Lone Architect Authority)

/// Represents a seat on the Aluminum OS Pantheon Council.
///
/// Each variant corresponds to a major technology organization invited to operate
/// its AI intelligences through the neutral Aluminum OS substrate. All seats are
/// equal in constitutional standing and are governed by the Net-Positive
/// Flourishing Metric (NPFM). No seat may override the foundational fiduciary
/// duty against busywork and extraction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CouncilSeat {
    /// Google — Search, Workspace, DeepMind AI research, Android
    Google,
    /// Tesla — Physical embodiment, autonomous vehicles, energy systems
    Tesla,
    /// Amazon — Cloud infrastructure (AWS), logistics, Alexa AI
    Amazon,
    /// Microsoft — Productivity ecosystem, Azure, GitHub, Copilot
    Microsoft,
    /// Anthropic — Constitutional AI research, Claude model family
    Anthropic,
    /// OpenAI — GPT model family, operator ecosystem
    OpenAI,
}

impl CouncilSeat {
    /// Returns all currently registered council seats.
    ///
    /// This list is the canonical registry of Pantheon Council members operating
    /// atop the Aluminum OS substrate. Adding a seat here does not grant any
    /// preferential access to the foundational ledger — all seats remain equal.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Google,
            Self::Tesla,
            Self::Amazon,
            Self::Microsoft,
            Self::Anthropic,
            Self::OpenAI,
        ]
    }

    /// Returns the human-readable display name for this council seat.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Google => "Google",
            Self::Tesla => "Tesla",
            Self::Amazon => "Amazon",
            Self::Microsoft => "Microsoft",
            Self::Anthropic => "Anthropic",
            Self::OpenAI => "OpenAI",
        }
    }
}

impl std::fmt::Display for CouncilSeat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_seats_are_registered() {
        let seats = CouncilSeat::all();
        assert_eq!(seats.len(), 6, "Expected 6 council seats");
    }

    #[test]
    fn seats_are_distinct() {
        let seats = CouncilSeat::all();
        let unique: std::collections::HashSet<_> = seats.iter().collect();
        assert_eq!(seats.len(), unique.len(), "All council seats must be distinct");
    }

    #[test]
    fn display_names_are_non_empty() {
        for seat in CouncilSeat::all() {
            assert!(
                !seat.display_name().is_empty(),
                "Display name must not be empty for {seat:?}"
            );
        }
    }
}
