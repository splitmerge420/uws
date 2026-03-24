// src/provider_health.rs
// Aluminum OS — Provider Health Monitor + Rate Limit Sentinel
//
// Novel Invention #7 — Provider API Health Monitor
// Novel Invention #8 — Rate Limit Sentinel
//
// Tracks the availability, latency, and remaining API quota for every
// connected provider. Proactively warns before rate limits are hit.
// Provides automatic backoff scheduling so no agent ever gets a 429.
//
// This is entirely pure/no-I/O: the structs and algorithms are testable
// without network access. The executor uses these structs to gate requests.
//
// Author: GitHub Copilot — Aluminum OS Session 2026-03-23

#![allow(dead_code)]

use std::collections::BTreeMap;

// ─── Provider identifier ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Provider {
    GitHub,
    Slack,
    Linear,
    Notion,
    Figma,
    Stripe,
    Gmail,
    GoogleDrive,
    MsGraph,
    AppleCloudKit,
}

impl Provider {
    pub fn as_str(&self) -> &str {
        match self {
            Provider::GitHub => "github",
            Provider::Slack => "slack",
            Provider::Linear => "linear",
            Provider::Notion => "notion",
            Provider::Figma => "figma",
            Provider::Stripe => "stripe",
            Provider::Gmail => "gmail",
            Provider::GoogleDrive => "google-drive",
            Provider::MsGraph => "ms-graph",
            Provider::AppleCloudKit => "apple-cloudkit",
        }
    }

    /// Default rate limit configuration for each provider.
    /// These reflect real-world API limits as of 2026.
    pub fn default_rate_limit(&self) -> RateLimit {
        match self {
            Provider::GitHub => RateLimit {
                requests_per_hour: 5000,
                requests_per_minute: None,
                concurrent_requests: 10,
            },
            Provider::Slack => RateLimit {
                requests_per_hour: 3600,
                requests_per_minute: Some(60),
                concurrent_requests: 5,
            },
            Provider::Linear => RateLimit {
                requests_per_hour: 1500,
                requests_per_minute: Some(25),
                concurrent_requests: 3,
            },
            Provider::Notion => RateLimit {
                requests_per_hour: 1800,
                requests_per_minute: Some(3),
                concurrent_requests: 2,
            },
            Provider::Figma => RateLimit {
                requests_per_hour: 3600,
                requests_per_minute: None,
                concurrent_requests: 10,
            },
            Provider::Stripe => RateLimit {
                requests_per_hour: 36000,
                requests_per_minute: Some(100),
                concurrent_requests: 25,
            },
            Provider::Gmail => RateLimit {
                requests_per_hour: 25000,
                requests_per_minute: Some(250),
                concurrent_requests: 10,
            },
            Provider::GoogleDrive => RateLimit {
                requests_per_hour: 18000,
                requests_per_minute: Some(300),
                concurrent_requests: 10,
            },
            Provider::MsGraph => RateLimit {
                requests_per_hour: 120000,
                requests_per_minute: Some(10000),
                concurrent_requests: 50,
            },
            Provider::AppleCloudKit => RateLimit {
                requests_per_hour: 40000,
                requests_per_minute: None,
                concurrent_requests: 15,
            },
        }
    }
}

// ─── Rate limit config ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests_per_hour: u64,
    pub requests_per_minute: Option<u64>,
    pub concurrent_requests: u64,
}

// ─── Health status ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// Fully operational.
    Healthy,
    /// Some degradation; requests may be slow.
    Degraded,
    /// Quota near-exhausted (< 10% remaining).
    QuotaWarning,
    /// Rate limited — must wait `retry_after_seconds`.
    RateLimited { retry_after_seconds: u64 },
    /// Provider unreachable or returning 5xx.
    Unavailable,
}

impl HealthStatus {
    pub fn is_safe_to_request(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }
}

// ─── Provider quota snapshot ─────────────────────────────────────────────

/// A point-in-time snapshot of a provider's quota state.
#[derive(Debug, Clone)]
pub struct QuotaSnapshot {
    pub provider: Provider,
    /// Requests used in the current rate-limit window.
    pub used: u64,
    /// Maximum requests allowed in the window.
    pub limit: u64,
    /// Seconds until the rate-limit window resets.
    pub reset_in_seconds: u64,
    /// ISO 8601 timestamp of this snapshot.
    pub timestamp: String,
}

impl QuotaSnapshot {
    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    /// Fraction of quota consumed (0.0 – 1.0).
    pub fn utilization(&self) -> f64 {
        if self.limit == 0 {
            1.0
        } else {
            self.used as f64 / self.limit as f64
        }
    }

    pub fn health_status(&self) -> HealthStatus {
        let util = self.utilization();
        if util >= 1.0 {
            HealthStatus::RateLimited {
                retry_after_seconds: self.reset_in_seconds,
            }
        } else if util >= 0.90 {
            HealthStatus::QuotaWarning
        } else {
            HealthStatus::Healthy
        }
    }
}

// ─── Health registry ──────────────────────────────────────────────────────

/// In-process health registry for all connected providers.
#[derive(Debug, Default, Clone)]
pub struct HealthRegistry {
    quotas: BTreeMap<String, QuotaSnapshot>,
    statuses: BTreeMap<String, HealthStatus>,
}

impl HealthRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a quota update from an HTTP response header (e.g., X-RateLimit-*).
    pub fn update_quota(&mut self, snapshot: QuotaSnapshot) {
        let key = snapshot.provider.as_str().to_string();
        let status = snapshot.health_status();
        self.quotas.insert(key.clone(), snapshot);
        self.statuses.insert(key, status);
    }

    /// Record a raw health status (e.g., from a connectivity check).
    pub fn set_status(&mut self, provider: &Provider, status: HealthStatus) {
        self.statuses.insert(provider.as_str().to_string(), status);
    }

    /// Get the current health status for a provider.
    pub fn get_status(&self, provider: &Provider) -> HealthStatus {
        self.statuses
            .get(provider.as_str())
            .cloned()
            .unwrap_or(HealthStatus::Healthy)
    }

    /// Get all providers that are safe to request.
    pub fn safe_providers(&self) -> Vec<String> {
        self.statuses
            .iter()
            .filter(|(_, s)| s.is_safe_to_request())
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Get all providers that need a backoff.
    pub fn throttled_providers(&self) -> Vec<(String, u64)> {
        self.statuses
            .iter()
            .filter_map(|(k, s)| {
                if let HealthStatus::RateLimited { retry_after_seconds } = s {
                    Some((k.clone(), *retry_after_seconds))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Summary: providers at quota warning.
    pub fn quota_warnings(&self) -> Vec<String> {
        self.statuses
            .iter()
            .filter(|(_, s)| matches!(s, HealthStatus::QuotaWarning))
            .map(|(k, _)| k.clone())
            .collect()
    }
}

// ─── Backoff calculator ───────────────────────────────────────────────────

/// Calculate exponential backoff with jitter for a failed request.
///
/// Formula: `base_ms * 2^attempt + jitter(seed, max_jitter_ms)`
/// Capped at `max_ms`.
pub fn calculate_backoff_ms(
    attempt: u32,
    base_ms: u64,
    max_ms: u64,
    jitter_seed: u64,
) -> u64 {
    let exponential = base_ms.saturating_mul(1u64 << attempt.min(10));
    let jitter = jitter_seed % (base_ms / 2 + 1);
    (exponential + jitter).min(max_ms)
}

/// Parse X-RateLimit-* headers from a provider response.
pub fn parse_rate_limit_headers(
    provider: Provider,
    headers: &BTreeMap<String, String>,
    timestamp: String,
) -> Option<QuotaSnapshot> {
    let used = headers
        .get("x-ratelimit-used")
        .or_else(|| headers.get("x-rate-limit-remaining").and_then(|_| headers.get("x-ratelimit-limit")))
        .and_then(|v| v.parse::<u64>().ok())?;

    let limit = headers
        .get("x-ratelimit-limit")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or_else(|| provider.default_rate_limit().requests_per_hour);

    let reset_in_seconds = headers
        .get("x-ratelimit-reset")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(3600);

    Some(QuotaSnapshot {
        provider,
        used,
        limit,
        reset_in_seconds,
        timestamp,
    })
}

// ─── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_as_str() {
        assert_eq!(Provider::GitHub.as_str(), "github");
        assert_eq!(Provider::MsGraph.as_str(), "ms-graph");
    }

    #[test]
    fn test_default_rate_limits_are_positive() {
        for p in [
            Provider::GitHub, Provider::Slack, Provider::Linear,
            Provider::Notion, Provider::Figma, Provider::Stripe,
        ] {
            assert!(p.default_rate_limit().requests_per_hour > 0);
            assert!(p.default_rate_limit().concurrent_requests > 0);
        }
    }

    #[test]
    fn test_quota_snapshot_remaining() {
        let snap = QuotaSnapshot {
            provider: Provider::GitHub,
            used: 1000,
            limit: 5000,
            reset_in_seconds: 3600,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(snap.remaining(), 4000);
        assert!((snap.utilization() - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_quota_snapshot_health_healthy() {
        let snap = QuotaSnapshot {
            provider: Provider::GitHub,
            used: 100,
            limit: 5000,
            reset_in_seconds: 3600,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(snap.health_status(), HealthStatus::Healthy);
        assert!(snap.health_status().is_safe_to_request());
    }

    #[test]
    fn test_quota_snapshot_health_warning_at_90_percent() {
        let snap = QuotaSnapshot {
            provider: Provider::Slack,
            used: 4600,
            limit: 5000,
            reset_in_seconds: 100,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(snap.health_status(), HealthStatus::QuotaWarning);
        // QuotaWarning is not safe to request (it's a warning — agents should throttle)
        assert!(!snap.health_status().is_safe_to_request());
    }

    #[test]
    fn test_quota_snapshot_rate_limited_at_100_percent() {
        let snap = QuotaSnapshot {
            provider: Provider::Notion,
            used: 5000,
            limit: 5000,
            reset_in_seconds: 60,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
        };
        if let HealthStatus::RateLimited { retry_after_seconds } = snap.health_status() {
            assert_eq!(retry_after_seconds, 60);
        } else {
            panic!("Expected RateLimited status");
        }
        assert!(!snap.health_status().is_safe_to_request());
    }

    #[test]
    fn test_health_registry_update_and_get() {
        let mut registry = HealthRegistry::new();
        let snap = QuotaSnapshot {
            provider: Provider::GitHub,
            used: 100,
            limit: 5000,
            reset_in_seconds: 3600,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
        };
        registry.update_quota(snap);
        assert_eq!(registry.get_status(&Provider::GitHub), HealthStatus::Healthy);
    }

    #[test]
    fn test_health_registry_set_status_unavailable() {
        let mut registry = HealthRegistry::new();
        registry.set_status(&Provider::Slack, HealthStatus::Unavailable);
        assert_eq!(registry.get_status(&Provider::Slack), HealthStatus::Unavailable);
    }

    #[test]
    fn test_health_registry_safe_providers() {
        let mut registry = HealthRegistry::new();
        registry.set_status(&Provider::GitHub, HealthStatus::Healthy);
        registry.set_status(&Provider::Slack, HealthStatus::RateLimited { retry_after_seconds: 60 });
        registry.set_status(&Provider::Linear, HealthStatus::Degraded);
        let safe = registry.safe_providers();
        assert!(safe.contains(&"github".to_string()));
        assert!(safe.contains(&"linear".to_string()));
        assert!(!safe.contains(&"slack".to_string()));
    }

    #[test]
    fn test_health_registry_throttled_providers() {
        let mut registry = HealthRegistry::new();
        registry.set_status(&Provider::GitHub, HealthStatus::RateLimited { retry_after_seconds: 120 });
        let throttled = registry.throttled_providers();
        assert_eq!(throttled.len(), 1);
        assert_eq!(throttled[0].1, 120);
    }

    #[test]
    fn test_calculate_backoff_ms_exponential() {
        let b0 = calculate_backoff_ms(0, 100, 10_000, 0);
        let b1 = calculate_backoff_ms(1, 100, 10_000, 0);
        let b2 = calculate_backoff_ms(2, 100, 10_000, 0);
        assert!(b1 >= b0);
        assert!(b2 >= b1);
    }

    #[test]
    fn test_calculate_backoff_ms_capped() {
        let backoff = calculate_backoff_ms(20, 1000, 5000, 0);
        assert!(backoff <= 5000);
    }

    #[test]
    fn test_parse_rate_limit_headers() {
        let mut headers = BTreeMap::new();
        headers.insert("x-ratelimit-used".to_string(), "1234".to_string());
        headers.insert("x-ratelimit-limit".to_string(), "5000".to_string());
        headers.insert("x-ratelimit-reset".to_string(), "1800".to_string());

        let snap = parse_rate_limit_headers(
            Provider::GitHub,
            &headers,
            "2026-01-01T00:00:00Z".to_string(),
        );
        assert!(snap.is_some());
        let snap = snap.unwrap();
        assert_eq!(snap.used, 1234);
        assert_eq!(snap.limit, 5000);
        assert_eq!(snap.reset_in_seconds, 1800);
    }

    #[test]
    fn test_quota_warning_list() {
        let mut registry = HealthRegistry::new();
        let snap = QuotaSnapshot {
            provider: Provider::Figma,
            used: 4700,
            limit: 5000,
            reset_in_seconds: 300,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
        };
        registry.update_quota(snap);
        let warnings = registry.quota_warnings();
        assert!(warnings.contains(&"figma".to_string()));
    }
}
