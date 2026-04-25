//! Provider interoperability skeleton.
//!
//! Phase 1 exposes provider homes and target boundaries.
//! Phase 1.5 synthesizes shared driver traits, auth, routing, observability,
//! and governance hooks across these homes.

pub mod cloud;
pub mod execution;
pub mod model;
pub mod workspace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Model,
    Cloud,
    Workspace,
    Execution,
}

#[derive(Debug, Clone)]
pub struct ProviderTarget {
    pub name: &'static str,
    pub kind: ProviderKind,
}
