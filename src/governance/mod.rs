//! House 12 governance runtime skeleton.
//!
//! Phase 1 exposes the full architecture as typed, compiling interfaces.
//! Enforcement is intentionally deferred to Phase 1.5 synthesis so the
//! execution spine can remain stable while governance attachment points are
//! made visible.
//!
//! This module converts the Grok House 12 Python attachment artifact into a
//! Rust-first skeleton. It avoids runtime monkey-patching and does not mutate
//! provider dispatch or Zero Trust behavior yet.

pub mod attachment_points;
pub mod dissent;
pub mod impact;
pub mod preflight;
pub mod priority;

pub use attachment_points::{GovernanceAttachmentPoint, GovernanceAttachmentTarget};
pub use dissent::DissentRecord;
pub use impact::{MetabolicImpact, MetabolicImpactRequirement};
pub use preflight::{FourStateClassifierGate, PreFlightResult, PreFlightState};
pub use priority::{House12Primitive, House12PriorityDecision, House12PriorityEngine};
