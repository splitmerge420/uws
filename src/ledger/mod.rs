/// Regenerative IP & Provenance Engine
///
/// Tracks the human/AI contribution split for every committing session and
/// records it as a verifiable `ProvenanceTrailer` that can be attached to a
/// git commit via `git notes`.
///
/// # CLI surface
/// - `uws ip sign`     — sign the current HEAD commit with a provenance trailer
/// - `uws ip monetize` — emit the registration payload for the Ledger API
///
/// # Upgrade path
/// The current signature uses SHA-256 over the canonical JSON representation
/// of the trailer fields.  A future release will replace this with Ed25519
/// (RFC 8032) or ML-DSA (FIPS 204) using the `ed25519-dalek` / `pqcrypto`
/// crates.  The `HitlSignature::algorithm` field already encodes the current
/// value (`"sha256-canonical-json"`) so consumers can negotiate the upgrade.
pub mod provenance;
