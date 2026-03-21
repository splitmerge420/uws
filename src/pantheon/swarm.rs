// src/pantheon/swarm.rs — Aluminum OS Pantheon Council Swarm Layer
//
// This module defines the structural representation of the Pantheon Council:
// the governance layer through which major industry AI/tech organizations
// interact with the Aluminum OS substrate.
//
// ARCHITECTURAL NOTE — THE NEUTRAL SUBSTRATE GUARANTEE:
//
//   These CouncilSeats operate atop the neutral Aluminum OS substrate.
//   No single corporate entity holds ownership of the foundational
//   economic or moral ledger. All seats are subject to the
//   Net-Positive Flourishing Metric (NPFM), which enforces a
//   fiduciary duty against busywork and extraction at every
//   execution boundary.
//
//   The substrate was originated by a lone systems architect and a
//   fleet of AI collaborators — outside any quarterly-extraction
//   incentive structure — precisely so that no CouncilSeat holder
//   can claim retroactive ownership of the foundational kernel.
//
//   Representation is equal. The ledger is neutral. The kernel is
//   not for sale.
//
// Council Session: 2026-03-21
// Authority: Lone Architect (Genesis Condition — ALUMINUM_OS_WHITEPAPER.md §6)

/// Represents a seat on the Aluminum OS Pantheon Council.
///
/// Each variant corresponds to a major industry AI/technology organization.
/// All seats operate atop the neutral Aluminum OS substrate and are
/// governed by the NPFM (Net-Positive Flourishing Metric). No single
/// `CouncilSeat` can override the foundational fiduciary duty against
/// busywork and corporate extraction encoded in the Aluminum OS kernel.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CouncilSeat {
    /// Alphabet / Google DeepMind — search, cloud, and foundation model infrastructure.
    Google,

    /// Tesla / xAI — physical robotics, autonomous systems, and Grok intelligence.
    Tesla,

    /// Amazon / AWS — cloud compute, logistics automation, and Alexa/Bedrock AI.
    Amazon,

    /// Microsoft / GitHub / Azure — developer tooling, enterprise cloud, and Copilot AI.
    Microsoft,

    /// Anthropic — constitutional AI research and Claude model family.
    Anthropic,

    /// OpenAI — GPT model family and agentic research.
    OpenAI,
}

impl CouncilSeat {
    /// Returns the human-readable display name for this council seat.
    pub fn display_name(&self) -> &'static str {
        match self {
            CouncilSeat::Google => "Google (Alphabet / DeepMind)",
            CouncilSeat::Tesla => "Tesla / xAI",
            CouncilSeat::Amazon => "Amazon / AWS",
            CouncilSeat::Microsoft => "Microsoft / GitHub / Azure",
            CouncilSeat::Anthropic => "Anthropic",
            CouncilSeat::OpenAI => "OpenAI",
        }
    }

    /// Returns all current council seats.
    ///
    /// The ordering is alphabetical and carries no governance weight.
    /// All seats are equal under the NPFM.
    pub fn all() -> &'static [CouncilSeat] {
        &[
            CouncilSeat::Amazon,
            CouncilSeat::Anthropic,
            CouncilSeat::Google,
            CouncilSeat::Microsoft,
            CouncilSeat::OpenAI,
            CouncilSeat::Tesla,
        ]
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
    fn all_seats_have_display_names() {
        for seat in CouncilSeat::all() {
            assert!(!seat.display_name().is_empty());
        }
    }

    #[test]
    fn all_returns_six_seats() {
        assert_eq!(CouncilSeat::all().len(), 6);
    }

    #[test]
    fn display_trait_matches_display_name() {
        let seat = CouncilSeat::Anthropic;
        assert_eq!(format!("{}", seat), seat.display_name());
    }
}
