// src/stripe_provider.rs
// Aluminum OS — Stripe Provider
//
// Exposes the Stripe REST API through the `uws stripe` command surface.
// Stripe is the canonical payment infrastructure; this provider gives
// AI agents and workflow automation structured JSON access to customers,
// invoices, charges, subscriptions, payment intents, and more.
//
// Command grammar:
//   uws stripe <resource> <method> [--params <JSON>] [--json <JSON>] [--dry-run]
//
// Examples:
//   uws stripe customers list --params '{"limit":"10"}'
//   uws stripe customers get --params '{"customer_id":"cus_123"}'
//   uws stripe invoices list --params '{"customer":"cus_123","status":"open"}'
//   uws stripe charges list --params '{"limit":"5"}'
//   uws stripe subscriptions list --params '{"status":"active"}'
//   uws stripe balance get
//   uws stripe products list
//
// Authentication:
//   Set STRIPE_SECRET_KEY or UWS_STRIPE_TOKEN in the environment.
//   Use the Stripe secret key from the Stripe Dashboard.
//
//   ⚠ WRITE OPERATIONS always require --dry-run first.
//   ⚠ Never commit your Stripe secret key to source control.
//
// Author: GitHub Copilot — Aluminum OS Session 2026-03-23

#![allow(dead_code)]

use std::collections::BTreeMap;

// ─── Service aliases ──────────────────────────────────────────────────────

pub const STRIPE_ALIASES: &[&str] = &["stripe"];

pub fn is_stripe_service(name: &str) -> bool {
    STRIPE_ALIASES.contains(&name)
}

// ─── API base ─────────────────────────────────────────────────────────────

pub const STRIPE_API_BASE: &str = "https://api.stripe.com/v1";

// ─── HTTP methods ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StripeHttpMethod {
    Get,
    Post,
    Delete,
}

impl StripeHttpMethod {
    pub fn as_str(&self) -> &str {
        match self {
            StripeHttpMethod::Get => "GET",
            StripeHttpMethod::Post => "POST",
            StripeHttpMethod::Delete => "DELETE",
        }
    }
}

// ─── Endpoint catalogue ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StripeEndpoint {
    pub resource: &'static str,
    pub method: &'static str,
    pub http_method: StripeHttpMethod,
    pub path_template: &'static str,
    pub requires_body: bool,
    pub description: &'static str,
    pub path_params: &'static [&'static str],
    /// If true, this is a destructive write — requires explicit --dry-run confirmation.
    pub is_destructive: bool,
}

pub const STRIPE_ENDPOINTS: &[StripeEndpoint] = &[
    // ── Balance ──────────────────────────────────────────────────────
    StripeEndpoint {
        resource: "balance",
        method: "get",
        http_method: StripeHttpMethod::Get,
        path_template: "/balance",
        requires_body: false,
        description: "Retrieve the current account balance",
        path_params: &[],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "balance",
        method: "transactions",
        http_method: StripeHttpMethod::Get,
        path_template: "/balance_transactions",
        requires_body: false,
        description: "List balance transactions",
        path_params: &[],
        is_destructive: false,
    },
    // ── Customers ────────────────────────────────────────────────────
    StripeEndpoint {
        resource: "customers",
        method: "list",
        http_method: StripeHttpMethod::Get,
        path_template: "/customers",
        requires_body: false,
        description: "List customers",
        path_params: &[],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "customers",
        method: "get",
        http_method: StripeHttpMethod::Get,
        path_template: "/customers/{customer_id}",
        requires_body: false,
        description: "Retrieve a customer by ID",
        path_params: &["customer_id"],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "customers",
        method: "create",
        http_method: StripeHttpMethod::Post,
        path_template: "/customers",
        requires_body: true,
        description: "Create a new customer",
        path_params: &[],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "customers",
        method: "update",
        http_method: StripeHttpMethod::Post,
        path_template: "/customers/{customer_id}",
        requires_body: true,
        description: "Update a customer",
        path_params: &["customer_id"],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "customers",
        method: "delete",
        http_method: StripeHttpMethod::Delete,
        path_template: "/customers/{customer_id}",
        requires_body: false,
        description: "Delete a customer",
        path_params: &["customer_id"],
        is_destructive: true,
    },
    StripeEndpoint {
        resource: "customers",
        method: "search",
        http_method: StripeHttpMethod::Get,
        path_template: "/customers/search",
        requires_body: false,
        description: "Search customers using Stripe query syntax",
        path_params: &[],
        is_destructive: false,
    },
    // ── Invoices ─────────────────────────────────────────────────────
    StripeEndpoint {
        resource: "invoices",
        method: "list",
        http_method: StripeHttpMethod::Get,
        path_template: "/invoices",
        requires_body: false,
        description: "List invoices",
        path_params: &[],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "invoices",
        method: "get",
        http_method: StripeHttpMethod::Get,
        path_template: "/invoices/{invoice_id}",
        requires_body: false,
        description: "Retrieve an invoice by ID",
        path_params: &["invoice_id"],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "invoices",
        method: "create",
        http_method: StripeHttpMethod::Post,
        path_template: "/invoices",
        requires_body: true,
        description: "Create an invoice",
        path_params: &[],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "invoices",
        method: "send",
        http_method: StripeHttpMethod::Post,
        path_template: "/invoices/{invoice_id}/send",
        requires_body: false,
        description: "Send an invoice to the customer",
        path_params: &["invoice_id"],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "invoices",
        method: "void",
        http_method: StripeHttpMethod::Post,
        path_template: "/invoices/{invoice_id}/void",
        requires_body: false,
        description: "Void an invoice",
        path_params: &["invoice_id"],
        is_destructive: true,
    },
    // ── Charges ──────────────────────────────────────────────────────
    StripeEndpoint {
        resource: "charges",
        method: "list",
        http_method: StripeHttpMethod::Get,
        path_template: "/charges",
        requires_body: false,
        description: "List charges",
        path_params: &[],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "charges",
        method: "get",
        http_method: StripeHttpMethod::Get,
        path_template: "/charges/{charge_id}",
        requires_body: false,
        description: "Retrieve a charge by ID",
        path_params: &["charge_id"],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "charges",
        method: "refund",
        http_method: StripeHttpMethod::Post,
        path_template: "/refunds",
        requires_body: true,
        description: "Create a refund for a charge",
        path_params: &[],
        is_destructive: true,
    },
    // ── Subscriptions ────────────────────────────────────────────────
    StripeEndpoint {
        resource: "subscriptions",
        method: "list",
        http_method: StripeHttpMethod::Get,
        path_template: "/subscriptions",
        requires_body: false,
        description: "List subscriptions",
        path_params: &[],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "subscriptions",
        method: "get",
        http_method: StripeHttpMethod::Get,
        path_template: "/subscriptions/{subscription_id}",
        requires_body: false,
        description: "Retrieve a subscription by ID",
        path_params: &["subscription_id"],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "subscriptions",
        method: "create",
        http_method: StripeHttpMethod::Post,
        path_template: "/subscriptions",
        requires_body: true,
        description: "Create a subscription",
        path_params: &[],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "subscriptions",
        method: "cancel",
        http_method: StripeHttpMethod::Delete,
        path_template: "/subscriptions/{subscription_id}",
        requires_body: false,
        description: "Cancel a subscription",
        path_params: &["subscription_id"],
        is_destructive: true,
    },
    // ── Payment Intents ───────────────────────────────────────────────
    StripeEndpoint {
        resource: "payment-intents",
        method: "list",
        http_method: StripeHttpMethod::Get,
        path_template: "/payment_intents",
        requires_body: false,
        description: "List payment intents",
        path_params: &[],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "payment-intents",
        method: "get",
        http_method: StripeHttpMethod::Get,
        path_template: "/payment_intents/{intent_id}",
        requires_body: false,
        description: "Retrieve a payment intent",
        path_params: &["intent_id"],
        is_destructive: false,
    },
    // ── Products ─────────────────────────────────────────────────────
    StripeEndpoint {
        resource: "products",
        method: "list",
        http_method: StripeHttpMethod::Get,
        path_template: "/products",
        requires_body: false,
        description: "List products",
        path_params: &[],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "products",
        method: "get",
        http_method: StripeHttpMethod::Get,
        path_template: "/products/{product_id}",
        requires_body: false,
        description: "Retrieve a product by ID",
        path_params: &["product_id"],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "products",
        method: "create",
        http_method: StripeHttpMethod::Post,
        path_template: "/products",
        requires_body: true,
        description: "Create a product",
        path_params: &[],
        is_destructive: false,
    },
    // ── Prices ───────────────────────────────────────────────────────
    StripeEndpoint {
        resource: "prices",
        method: "list",
        http_method: StripeHttpMethod::Get,
        path_template: "/prices",
        requires_body: false,
        description: "List prices",
        path_params: &[],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "prices",
        method: "create",
        http_method: StripeHttpMethod::Post,
        path_template: "/prices",
        requires_body: true,
        description: "Create a price",
        path_params: &[],
        is_destructive: false,
    },
    // ── Events ───────────────────────────────────────────────────────
    StripeEndpoint {
        resource: "events",
        method: "list",
        http_method: StripeHttpMethod::Get,
        path_template: "/events",
        requires_body: false,
        description: "List Stripe events (webhook logs)",
        path_params: &[],
        is_destructive: false,
    },
    StripeEndpoint {
        resource: "events",
        method: "get",
        http_method: StripeHttpMethod::Get,
        path_template: "/events/{event_id}",
        requires_body: false,
        description: "Retrieve an event by ID",
        path_params: &["event_id"],
        is_destructive: false,
    },
];

// ─── Error type ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum StripeError {
    EndpointNotFound { resource: String, method: String },
    TokenNotFound,
    DestructiveOperationRequiresDryRun { resource: String, method: String },
    ParseError(String),
}

impl std::fmt::Display for StripeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StripeError::EndpointNotFound { resource, method } => {
                write!(f, "Unknown Stripe endpoint: {resource} {method}")
            }
            StripeError::TokenNotFound => write!(
                f,
                "Stripe secret key not found. Set STRIPE_SECRET_KEY or UWS_STRIPE_TOKEN."
            ),
            StripeError::DestructiveOperationRequiresDryRun { resource, method } => {
                write!(
                    f,
                    "'{resource} {method}' is destructive. Use --dry-run first to preview."
                )
            }
            StripeError::ParseError(s) => write!(f, "Parse error: {s}"),
        }
    }
}

// ─── Token resolution ─────────────────────────────────────────────────────

pub fn resolve_stripe_token() -> Option<String> {
    std::env::var("UWS_STRIPE_TOKEN")
        .ok()
        .or_else(|| std::env::var("STRIPE_SECRET_KEY").ok())
}

// ─── Path parameter substitution ─────────────────────────────────────────

pub fn build_url(
    path_template: &str,
    params: &BTreeMap<String, String>,
) -> (String, BTreeMap<String, String>) {
    let mut path = path_template.to_string();
    let mut remaining = BTreeMap::new();

    for (k, v) in params {
        let token = format!("{{{k}}}");
        if path.contains(&token) {
            path = path.replace(&token, v);
        } else {
            remaining.insert(k.clone(), v.clone());
        }
    }

    (format!("{STRIPE_API_BASE}{path}"), remaining)
}

// ─── Request builder ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StripeRequest {
    pub http_method: String,
    pub url: String,
    pub query_params: BTreeMap<String, String>,
    pub body: Option<BTreeMap<String, String>>,
    pub auth_header: String,
    pub is_destructive: bool,
    pub dry_run: bool,
}

/// Build a Stripe request.  Returns Err if the endpoint is destructive and
/// `dry_run` is false, enforcing the mandatory dry-run gate.
pub fn build_request(
    endpoint: &StripeEndpoint,
    params: BTreeMap<String, String>,
    body: Option<BTreeMap<String, String>>,
    token: &str,
    dry_run: bool,
) -> Result<StripeRequest, StripeError> {
    if endpoint.is_destructive && !dry_run {
        return Err(StripeError::DestructiveOperationRequiresDryRun {
            resource: endpoint.resource.to_string(),
            method: endpoint.method.to_string(),
        });
    }

    let (url, query_params) = build_url(endpoint.path_template, &params);
    let auth_token = if dry_run { "[REDACTED]".to_string() } else { token.to_string() };

    Ok(StripeRequest {
        http_method: endpoint.http_method.as_str().to_string(),
        url,
        query_params,
        body,
        auth_header: format!("Bearer {auth_token}"),
        is_destructive: endpoint.is_destructive,
        dry_run,
    })
}

// ─── Endpoint lookup ──────────────────────────────────────────────────────

pub fn find_endpoint(resource: &str, method: &str) -> Option<&'static StripeEndpoint> {
    STRIPE_ENDPOINTS
        .iter()
        .find(|e| e.resource == resource && e.method == method)
}

// ─── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_stripe_service() {
        assert!(is_stripe_service("stripe"));
        assert!(!is_stripe_service("figma"));
        assert!(!is_stripe_service("github"));
    }

    #[test]
    fn test_endpoint_catalogue_nonempty() {
        assert!(!STRIPE_ENDPOINTS.is_empty());
    }

    #[test]
    fn test_find_balance_get() {
        let ep = find_endpoint("balance", "get").unwrap();
        assert_eq!(ep.http_method, StripeHttpMethod::Get);
        assert!(!ep.is_destructive);
    }

    #[test]
    fn test_find_customers_delete_is_destructive() {
        let ep = find_endpoint("customers", "delete").unwrap();
        assert!(ep.is_destructive);
    }

    #[test]
    fn test_destructive_without_dry_run_returns_error() {
        let ep = find_endpoint("customers", "delete").unwrap();
        let mut params = BTreeMap::new();
        params.insert("customer_id".to_string(), "cus_123".to_string());
        let result = build_request(ep, params, None, "sk_live_token", false);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("destructive"));
    }

    #[test]
    fn test_destructive_with_dry_run_succeeds() {
        let ep = find_endpoint("customers", "delete").unwrap();
        let mut params = BTreeMap::new();
        params.insert("customer_id".to_string(), "cus_123".to_string());
        let result = build_request(ep, params, None, "sk_live_token", true);
        assert!(result.is_ok());
        let req = result.unwrap();
        assert!(req.auth_header.contains("[REDACTED]"));
    }

    #[test]
    fn test_build_url_substitutes_path_params() {
        let mut params = BTreeMap::new();
        params.insert("invoice_id".to_string(), "inv_123".to_string());
        let (url, _) = build_url("/invoices/{invoice_id}", &params);
        assert_eq!(url, "https://api.stripe.com/v1/invoices/inv_123");
    }

    #[test]
    fn test_non_destructive_without_dry_run_succeeds() {
        let ep = find_endpoint("customers", "list").unwrap();
        let result = build_request(ep, BTreeMap::new(), None, "sk_test_token", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_display_token_not_found() {
        let e = StripeError::TokenNotFound;
        assert!(e.to_string().contains("STRIPE_SECRET_KEY"));
    }

    #[test]
    fn test_error_display_destructive() {
        let e = StripeError::DestructiveOperationRequiresDryRun {
            resource: "subscriptions".to_string(),
            method: "cancel".to_string(),
        };
        assert!(e.to_string().contains("subscriptions cancel"));
        assert!(e.to_string().contains("--dry-run"));
    }

    #[test]
    fn test_invoices_endpoints_present() {
        assert!(find_endpoint("invoices", "list").is_some());
        assert!(find_endpoint("invoices", "send").is_some());
        assert!(find_endpoint("invoices", "void").is_some());
    }

    #[test]
    fn test_subscriptions_endpoints_present() {
        assert!(find_endpoint("subscriptions", "list").is_some());
        assert!(find_endpoint("subscriptions", "cancel").is_some());
    }

    #[test]
    fn test_events_endpoints_present() {
        assert!(find_endpoint("events", "list").is_some());
        assert!(find_endpoint("events", "get").is_some());
    }

    #[test]
    fn test_stripe_api_base() {
        assert_eq!(STRIPE_API_BASE, "https://api.stripe.com/v1");
    }

    #[test]
    fn test_unique_resource_method_pairs() {
        let mut seen = std::collections::HashSet::new();
        for ep in STRIPE_ENDPOINTS {
            let key = format!("{}/{}", ep.resource, ep.method);
            assert!(seen.insert(key.clone()), "Duplicate endpoint: {key}");
        }
    }
}
