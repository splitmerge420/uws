// src/drivers/mod.rs
// Aluminum OS — Provider Driver Registry
//
// Exposes all native provider drivers that implement the ProviderDriver trait.

/// FHIR R4 Health & Wellness driver (Domain 1).
pub mod health_fhir;

/// Notion workspace driver — replaces Zapier (Domain 3).
pub mod notion;
