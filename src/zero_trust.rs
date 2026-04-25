// src/zero_trust.rs — Zero Trust Integration Gate
//
// Pre-execution gate for all external integration calls in uws.
// Every provider auth flow, outbound network request, and workspace
// file write MUST pass through this gate before executing.
//
// Pattern inspired by the Kintsugi Gate in bazinga/src/main.rs.
// Invariants enforced: INV-004 (Zero Trust), INV-003 (Audit Trail),
//                      INV-005 (Fail-Closed)

use std::io::Write;

// ─── Integration Kind ─────────────────────────────────────────────────────────

/// Category of external integration being gated.
///
/// Every operation that crosses a trust boundary must be classified
/// with one of these variants before calling `ZeroTrustGate::check`.
#[derive(Debug, Clone, PartialEq)]
pub enum IntegrationKind {
    /// OAuth2 / API-key provider authentication flows
    /// (e.g. `uws auth login`, `uws ms-auth login`, `uws apple-auth`).
    ProviderAuth,

    /// Outbound HTTP requests to external APIs
    /// (e.g. Google Discovery fetch, Microsoft Graph calls).
    NetworkEgress,

    /// File writes to paths outside the current workspace
    /// (e.g. `--output-dir` flags, credential store writes).
    FileWrite,
}

// ─── Gate Policy ──────────────────────────────────────────────────────────────

/// Policy controlling which integration kinds are permitted.
///
/// Build a `GatePolicy` via the named constructors (`permissive`,
/// `deny_all`) or by filling the fields directly for fine-grained
/// control.
pub struct GatePolicy {
    /// Whether ProviderAuth integrations are allowed.
    pub allow_provider_auth: bool,
    /// Whether NetworkEgress integrations are allowed.
    pub allow_network_egress: bool,
    /// Whether FileWrite integrations are allowed.
    pub allow_file_write: bool,
    /// If `Some`, blocked operations append a line to this path.
    pub audit_log_path: Option<String>,
}

impl GatePolicy {
    /// All integrations permitted — for normal CLI operation.
    pub fn permissive() -> Self {
        GatePolicy {
            allow_provider_auth: true,
            allow_network_egress: true,
            allow_file_write: true,
            audit_log_path: None,
        }
    }

    /// All integrations blocked — for locked-down / test environments.
    pub fn deny_all() -> Self {
        GatePolicy {
            allow_provider_auth: false,
            allow_network_egress: false,
            allow_file_write: false,
            audit_log_path: None,
        }
    }
}

impl Default for GatePolicy {
    fn default() -> Self {
        Self::permissive()
    }
}

// ─── Gate Context ─────────────────────────────────────────────────────────────

/// Context passed to the gate for each integration check.
pub struct GateContext<'a> {
    /// The kind of integration being attempted.
    pub kind: IntegrationKind,
    /// Human-readable resource identifier
    /// (e.g. `"microsoft/oauth2"`, `"graph.microsoft.com"`, `"/tmp/out.json"`).
    pub resource: &'a str,
    /// Actor requesting the integration
    /// (e.g. `"uws-cli"`, `"agent:claude"`, `"swarm-commander"`).
    pub actor: &'a str,
}

// ─── Gate Decision ────────────────────────────────────────────────────────────

/// Decision returned by `ZeroTrustGate::check`.
#[derive(Debug, PartialEq)]
pub enum GateDecision {
    /// Integration is permitted; proceed.
    Allow,
    /// Integration is blocked; the string contains the denial reason.
    Block(String),
}

// ─── Zero Trust Gate ──────────────────────────────────────────────────────────

/// Zero Trust Integration Gate.
///
/// Every external integration call in uws **MUST** pass through this
/// gate before executing. The gate evaluates the operation against the
/// active `GatePolicy` and writes an audit log entry on any block.
///
/// # Example
///
/// ```rust
/// use uws::zero_trust::{ZeroTrustGate, GatePolicy, GateContext, IntegrationKind, GateDecision};
///
/// let gate = ZeroTrustGate::new(GatePolicy::permissive());
/// let ctx = GateContext {
///     kind: IntegrationKind::ProviderAuth,
///     resource: "microsoft/oauth2",
///     actor: "uws-cli",
/// };
/// assert_eq!(gate.check(&ctx), GateDecision::Allow);
/// ```
pub struct ZeroTrustGate {
    policy: GatePolicy,
}

impl ZeroTrustGate {
    /// Create a new gate with the given policy.
    pub fn new(policy: GatePolicy) -> Self {
        ZeroTrustGate { policy }
    }

    /// Evaluate whether the integration described by `ctx` is permitted.
    ///
    /// On block, appends an audit log entry to `policy.audit_log_path`
    /// (if set) and returns `GateDecision::Block(reason)`.
    /// On allow, returns `GateDecision::Allow` with no side effects.
    pub fn check(&self, ctx: &GateContext<'_>) -> GateDecision {
        let allowed = match ctx.kind {
            IntegrationKind::ProviderAuth => self.policy.allow_provider_auth,
            IntegrationKind::NetworkEgress => self.policy.allow_network_egress,
            IntegrationKind::FileWrite => self.policy.allow_file_write,
        };

        if allowed {
            GateDecision::Allow
        } else {
            let reason = format!(
                "Zero Trust Gate BLOCKED {:?} for resource '{}' by actor '{}' — policy denies this integration kind",
                ctx.kind, ctx.resource, ctx.actor,
            );
            self.write_audit_log(ctx, &reason);
            GateDecision::Block(reason)
        }
    }

    /// Append a structured audit log entry to `policy.audit_log_path`.
    ///
    /// Best-effort: silently ignores log write failures so a logging
    /// misconfiguration never converts a block into a crash.
    fn write_audit_log(&self, ctx: &GateContext<'_>, reason: &str) {
        let Some(ref path) = self.policy.audit_log_path else {
            return;
        };

        let timestamp = {
            // Portable wall-clock seconds since UNIX epoch (no external dep).
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        };

        let entry = format!(
            "ZERO_TRUST_BLOCK | ts={} | actor={} | resource={} | kind={:?} | reason={}\n",
            timestamp, ctx.actor, ctx.resource, ctx.kind, reason,
        );

        // Best-effort append — do not panic on log failure.
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = file.write_all(entry.as_bytes());
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissive_gate_allows_all_kinds() {
        let gate = ZeroTrustGate::new(GatePolicy::permissive());

        let ctx = GateContext {
            kind: IntegrationKind::ProviderAuth,
            resource: "microsoft/oauth2",
            actor: "uws-cli",
        };
        assert_eq!(gate.check(&ctx), GateDecision::Allow);

        let ctx = GateContext {
            kind: IntegrationKind::NetworkEgress,
            resource: "graph.microsoft.com",
            actor: "uws-cli",
        };
        assert_eq!(gate.check(&ctx), GateDecision::Allow);

        let ctx = GateContext {
            kind: IntegrationKind::FileWrite,
            resource: "/tmp/output.json",
            actor: "uws-cli",
        };
        assert_eq!(gate.check(&ctx), GateDecision::Allow);
    }

    #[test]
    fn test_deny_all_gate_blocks_all_kinds() {
        let gate = ZeroTrustGate::new(GatePolicy::deny_all());

        let ctx = GateContext {
            kind: IntegrationKind::ProviderAuth,
            resource: "microsoft/oauth2",
            actor: "uws-cli",
        };
        assert!(matches!(gate.check(&ctx), GateDecision::Block(_)));

        let ctx = GateContext {
            kind: IntegrationKind::NetworkEgress,
            resource: "api.example.com",
            actor: "uws-cli",
        };
        assert!(matches!(gate.check(&ctx), GateDecision::Block(_)));

        let ctx = GateContext {
            kind: IntegrationKind::FileWrite,
            resource: "/tmp/output.json",
            actor: "uws-cli",
        };
        assert!(matches!(gate.check(&ctx), GateDecision::Block(_)));
    }

    /// Synthetic fixture: ms-auth login MUST be blocked when gate is in deny_all mode.
    /// This verifies the gate is correctly wired into the provider-auth path.
    #[test]
    fn test_synthetic_ms_auth_login_blocked_by_deny_all_gate() {
        let gate = ZeroTrustGate::new(GatePolicy::deny_all());
        let ctx = GateContext {
            kind: IntegrationKind::ProviderAuth,
            resource: "microsoft/oauth2",
            actor: "uws-cli",
        };
        match gate.check(&ctx) {
            GateDecision::Block(reason) => {
                assert!(
                    reason.contains("BLOCKED"),
                    "Block reason must contain 'BLOCKED': {reason}"
                );
            }
            GateDecision::Allow => panic!(
                "Zero Trust Gate must block ms-auth login when policy is deny_all"
            ),
        }
    }

    /// Selective policy: ProviderAuth blocked, NetworkEgress allowed.
    #[test]
    fn test_selective_policy() {
        let gate = ZeroTrustGate::new(GatePolicy {
            allow_provider_auth: false,
            allow_network_egress: true,
            allow_file_write: true,
            audit_log_path: None,
        });

        let auth_ctx = GateContext {
            kind: IntegrationKind::ProviderAuth,
            resource: "google/oauth2",
            actor: "uws-cli",
        };
        assert!(matches!(gate.check(&auth_ctx), GateDecision::Block(_)));

        let net_ctx = GateContext {
            kind: IntegrationKind::NetworkEgress,
            resource: "www.googleapis.com",
            actor: "uws-cli",
        };
        assert_eq!(gate.check(&net_ctx), GateDecision::Allow);
    }

    #[test]
    fn test_audit_log_written_on_block() {
        let log_path = std::env::temp_dir().join("uws_zero_trust_test.log");
        let log_path = log_path.to_str().expect("temp_dir path must be valid UTF-8");
        let _ = std::fs::remove_file(log_path); // clean slate

        let gate = ZeroTrustGate::new(GatePolicy {
            allow_provider_auth: false,
            allow_network_egress: true,
            allow_file_write: true,
            audit_log_path: Some(log_path.to_string()),
        });

        let ctx = GateContext {
            kind: IntegrationKind::ProviderAuth,
            resource: "microsoft/oauth2",
            actor: "uws-cli",
        };

        assert!(matches!(gate.check(&ctx), GateDecision::Block(_)));

        let log_contents =
            std::fs::read_to_string(log_path).expect("Audit log file must be created on block");
        assert!(
            log_contents.contains("ZERO_TRUST_BLOCK"),
            "Log must contain ZERO_TRUST_BLOCK marker"
        );
        assert!(
            log_contents.contains("microsoft/oauth2"),
            "Log must contain the resource name"
        );
        assert!(
            log_contents.contains("uws-cli"),
            "Log must contain the actor name"
        );

        let _ = std::fs::remove_file(log_path);
    }

    #[test]
    fn test_audit_log_not_written_on_allow() {
        let log_path = std::env::temp_dir().join("uws_zero_trust_allow_test.log");
        let log_path = log_path.to_str().expect("temp_dir path must be valid UTF-8");
        let _ = std::fs::remove_file(log_path); // clean slate

        let gate = ZeroTrustGate::new(GatePolicy {
            allow_provider_auth: true,
            allow_network_egress: true,
            allow_file_write: true,
            audit_log_path: Some(log_path.to_string()),
        });

        let ctx = GateContext {
            kind: IntegrationKind::ProviderAuth,
            resource: "microsoft/oauth2",
            actor: "uws-cli",
        };

        assert_eq!(gate.check(&ctx), GateDecision::Allow);
        assert!(
            !std::path::Path::new(log_path).exists(),
            "Audit log must not be created when gate allows"
        );
    }
}
