/// Targets where governance can be attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceAttachmentTarget {
    ZeroTrustGate,
    ProviderDispatch,
}

/// Represents an attachment point in the system.
pub struct GovernanceAttachmentPoint {
    pub target: GovernanceAttachmentTarget,
}

impl GovernanceAttachmentPoint {
    pub fn new(target: GovernanceAttachmentTarget) -> Self {
        Self { target }
    }
}

/// Attach governance to Zero Trust gate.
///
/// Phase 1: no-op.
/// Phase 1.5: integrate PreFlightGate + PriorityEngine.
pub fn attach_to_zero_trust() {
    // TODO: integrate House 12 preflight + priority
}

/// Attach governance to provider dispatch.
///
/// Phase 1: no-op.
/// Phase 1.5: integrate consent + priority + metabolic impact.
pub fn attach_to_provider_dispatch() {
    // TODO: integrate governance runtime hooks
}
