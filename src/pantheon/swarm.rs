//! `swarm.rs` — Council seat definitions for the Aluminum OS governance layer.
//!
//! The [`CouncilSeat`] enum is the canonical registry of entities that hold seats
//! on the Aluminum Council. All seats operate atop the neutral substrate governed
//! by the Neutral Provider Fiduciary Mandate (NPFM). No single seat can override
//! the foundational fiduciary duty owed to users across all platforms.
//!
//! Council seats grant visibility, deliberation voice, and provider accountability.
//! They do **not** grant control over the royalty oracle, the provenance ledger, or
//! the Constitutional Invariants.

use std::fmt;

/// Canonical registry of Aluminum Council seats.
///
/// Each variant represents a corporate entity that holds a seat on the Council.
/// Seat-holders may participate in deliberation and are accountable under the NPFM
/// for the behavior of their provider driver. No seat-holder can override the
/// foundational fiduciary duty — the substrate is neutral by construction because
/// it was built outside any of these organisations.
///
/// See `whitepaper/ALUMINUM_OS_WHITEPAPER.md`, Section 6.3 for the full governance
/// rationale.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CouncilSeat {
    /// Google LLC — provider of Google Workspace (Gmail, Drive, Calendar, …).
    Google,
    /// Tesla, Inc. — energy, autonomy, and manufacturing seat.
    Tesla,
    /// Amazon.com, Inc. — cloud infrastructure and logistics seat.
    Amazon,
    /// Microsoft Corporation — provider of Microsoft 365 (Outlook, OneDrive, Teams, …).
    Microsoft,
    /// Anthropic PBC — provider of the Claude constitutional AI family.
    Anthropic,
    /// OpenAI, LLC — provider of the GPT model family.
    OpenAI,
}

impl CouncilSeat {
    /// Returns the human-readable display name for this seat.
    ///
    /// # Examples
    ///
    /// ```
    /// use uws::pantheon::swarm::CouncilSeat;
    /// assert_eq!(CouncilSeat::Anthropic.display_name(), "Anthropic");
    /// ```
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

    /// Returns all Council seats in canonical order.
    ///
    /// # Examples
    ///
    /// ```
    /// use uws::pantheon::swarm::CouncilSeat;
    /// assert_eq!(CouncilSeat::all().len(), 6);
    /// ```
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

impl fmt::Display for CouncilSeat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_matches_variant() {
        assert_eq!(CouncilSeat::Google.display_name(), "Google");
        assert_eq!(CouncilSeat::Tesla.display_name(), "Tesla");
        assert_eq!(CouncilSeat::Amazon.display_name(), "Amazon");
        assert_eq!(CouncilSeat::Microsoft.display_name(), "Microsoft");
        assert_eq!(CouncilSeat::Anthropic.display_name(), "Anthropic");
        assert_eq!(CouncilSeat::OpenAI.display_name(), "OpenAI");
    }

    #[test]
    fn display_impl_uses_display_name() {
        for seat in CouncilSeat::all() {
            assert_eq!(format!("{seat}"), seat.display_name());
        }
    }

    #[test]
    fn all_returns_six_seats() {
        assert_eq!(CouncilSeat::all().len(), 6);
    }

    #[test]
    fn all_contains_every_variant() {
        let all = CouncilSeat::all();
        assert!(all.contains(&CouncilSeat::Google));
        assert!(all.contains(&CouncilSeat::Tesla));
        assert!(all.contains(&CouncilSeat::Amazon));
        assert!(all.contains(&CouncilSeat::Microsoft));
        assert!(all.contains(&CouncilSeat::Anthropic));
        assert!(all.contains(&CouncilSeat::OpenAI));
    }

    #[test]
    fn clone_and_eq() {
        let seat = CouncilSeat::Anthropic;
        let cloned = seat.clone();
        assert_eq!(seat, cloned);
    }

    #[test]
    fn hash_works() {
        use std::collections::HashSet;
        let set: HashSet<CouncilSeat> = CouncilSeat::all().iter().cloned().collect();
        assert_eq!(set.len(), 6);
    }

    #[test]
    fn no_seat_is_privileged() {
        // All seats are equal peers under the NPFM — no seat has a special ordinal.
        // This test is intentionally simple: it documents the invariant in the test suite.
        for seat in CouncilSeat::all() {
            assert!(!seat.display_name().is_empty());
        }
    }
}
