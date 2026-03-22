// src/github_provider.rs — Universal Workspace CLI (uws)
// GitHub as a first-class uws provider
//
// Surfaces the GitHub REST + GraphQL APIs through the same uws grammar:
//   uws github issues list --params '{"owner":"acme","repo":"api","state":"open"}'
//   uws github pulls list  --params '{"owner":"acme","repo":"api"}'
//   uws github models list
//   uws github copilot chat --json '{"prompt":"explain this code"}'
//   uws github actions runs list --params '{"owner":"acme","repo":"api"}'
//   uws github search code --params '{"q":"JanusRouter language:rust"}'
//   uws github releases latest --params '{"owner":"acme","repo":"api"}'
//
// Authentication: GITHUB_TOKEN environment variable (Personal Access Token or
// workflow token injected automatically in GitHub Actions).
//
// GitHub benefits from this provider by:
//   1. GitHub Models gets more users through `uws github models list`
//   2. GitHub Actions CI workflows can use `uws github` for self-service automation
//   3. GitHub Copilot Chat becomes the default `uws ai` backend when no other key present
//   4. Issues + PRs become part of the universal workspace, searchable alongside email/calendar
//   5. Actions job summaries can include workspace data alongside code
//
// Licensed under the Apache License, Version 2.0

// Note: This module is intentionally free of external crate dependencies
// (no reqwest/serde_json) so it compiles with the current minimal Cargo.toml.
// HTTP calls are documented as the request shape; the executor module builds
// the actual requests at runtime using the same pattern as other providers.

/// GitHub REST API base URL
pub const GITHUB_API_BASE: &str = "https://api.github.com";

/// GitHub GraphQL API endpoint
pub const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";

/// GitHub Models inference endpoint (compatible with OpenAI SDK)
pub const GITHUB_MODELS_URL: &str = "https://models.inference.ai.azure.com";

/// GitHub Copilot Chat endpoint (GitHub Copilot API)
pub const GITHUB_COPILOT_CHAT_URL: &str = "https://api.githubcopilot.com/chat/completions";

/// Environment variable carrying the GitHub token
pub const GITHUB_TOKEN_ENV: &str = "GITHUB_TOKEN";

// ─── Service Registry ─────────────────────────────────────────

/// A GitHub service registered in the uws command surface.
pub struct GitHubServiceEntry {
    /// Command aliases, e.g. &["github-issues", "gh-issues"]
    pub aliases: &'static [&'static str],
    /// The REST API path template, e.g. "/repos/{owner}/{repo}/issues"
    pub api_path: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Required GitHub token scopes
    pub scopes: &'static [&'static str],
    /// Whether this endpoint supports pagination via Link headers
    pub paginated: bool,
}

/// All uws-exposed GitHub services.
pub const GITHUB_SERVICES: &[GitHubServiceEntry] = &[
    // ── Issues ──────────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-issues", "gh-issues"],
        api_path: "/repos/{owner}/{repo}/issues",
        description: "GitHub Issues: list, get, create, update, close",
        scopes: &["repo"],
        paginated: true,
    },
    // ── Pull Requests ────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-pulls", "github-prs", "gh-pulls"],
        api_path: "/repos/{owner}/{repo}/pulls",
        description: "GitHub Pull Requests: list, get, create, merge",
        scopes: &["repo"],
        paginated: true,
    },
    // ── Actions ──────────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-actions", "gh-actions"],
        api_path: "/repos/{owner}/{repo}/actions/runs",
        description: "GitHub Actions: list runs, get logs, trigger workflows",
        scopes: &["actions:read"],
        paginated: true,
    },
    // ── Releases ─────────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-releases", "gh-releases"],
        api_path: "/repos/{owner}/{repo}/releases",
        description: "GitHub Releases: list, get, create, upload assets",
        scopes: &["contents:read"],
        paginated: true,
    },
    // ── Code Search ──────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-search", "gh-search"],
        api_path: "/search/code",
        description: "GitHub Code Search: search across all public repositories",
        scopes: &[],
        paginated: true,
    },
    // ── Repositories ─────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-repos", "gh-repos"],
        api_path: "/user/repos",
        description: "GitHub Repositories: list, create, get, fork, delete",
        scopes: &["repo"],
        paginated: true,
    },
    // ── GitHub Models (AI inference) ─────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-models", "gh-models"],
        api_path: "/marketplace/models",  // GitHub Models catalogue
        description: "GitHub Models: list available AI models, run inference",
        scopes: &["models:read"],
        paginated: false,
    },
    // ── Notifications ────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-notifications", "gh-notifications"],
        api_path: "/notifications",
        description: "GitHub Notifications: list, mark as read, manage subscriptions",
        scopes: &["notifications"],
        paginated: true,
    },
    // ── Gists ────────────────────────────────────────────────────
    GitHubServiceEntry {
        aliases: &["github-gists", "gh-gists"],
        api_path: "/gists",
        description: "GitHub Gists: list, create, update, fork, star",
        scopes: &["gist"],
        paginated: true,
    },
];

// ─── GitHub Models catalogue ──────────────────────────────────

/// A model available through GitHub Models.
#[derive(Debug, Clone)]
pub struct GitHubModel {
    pub id: &'static str,
    pub display_name: &'static str,
    pub publisher: &'static str,
    pub description: &'static str,
    /// Whether this model is free-tier eligible
    pub free_tier: bool,
}

/// Catalogue of models available via GitHub Models (as of March 2026).
/// Full list: https://github.com/marketplace/models
pub const GITHUB_MODELS: &[GitHubModel] = &[
    GitHubModel {
        id: "gpt-4o",
        display_name: "GPT-4o",
        publisher: "OpenAI",
        description: "Most capable OpenAI model, optimised for speed and quality",
        free_tier: true,
    },
    GitHubModel {
        id: "gpt-4o-mini",
        display_name: "GPT-4o mini",
        publisher: "OpenAI",
        description: "Fast, affordable GPT-4o variant for simple tasks",
        free_tier: true,
    },
    GitHubModel {
        id: "meta-llama-3.1-70b-instruct",
        display_name: "Llama 3.1 70B Instruct",
        publisher: "Meta",
        description: "Open-weight instruction-tuned model from Meta",
        free_tier: true,
    },
    GitHubModel {
        id: "meta-llama-3.1-405b-instruct",
        display_name: "Llama 3.1 405B Instruct",
        publisher: "Meta",
        description: "Meta's largest open model",
        free_tier: false,
    },
    GitHubModel {
        id: "mistral-large",
        display_name: "Mistral Large",
        publisher: "Mistral AI",
        description: "Mistral's flagship model for complex reasoning",
        free_tier: true,
    },
    GitHubModel {
        id: "ai21-jamba-1.5-large",
        display_name: "Jamba 1.5 Large",
        publisher: "AI21 Labs",
        description: "Long-context hybrid SSM-Transformer model",
        free_tier: false,
    },
    GitHubModel {
        id: "cohere-command-r-plus",
        display_name: "Command R+",
        publisher: "Cohere",
        description: "RAG-optimised model from Cohere",
        free_tier: true,
    },
    GitHubModel {
        id: "phi-3-medium-128k-instruct",
        display_name: "Phi-3 Medium (128k)",
        publisher: "Microsoft",
        description: "Microsoft's small but powerful Phi-3 model",
        free_tier: true,
    },
];

// ─── Authentication helper ────────────────────────────────────

/// Returns the GitHub token from the environment, in priority order:
/// 1. GITHUB_TOKEN (set by GitHub Actions automatically)
/// 2. GH_TOKEN (set by the gh CLI)
/// 3. GITHUB_PAT (legacy variable name)
///
/// Returns None if no token is found.
pub fn github_token() -> Option<String> {
    for var in &["GITHUB_TOKEN", "GH_TOKEN", "GITHUB_PAT"] {
        if let Ok(v) = std::env::var(var) {
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    None
}

/// Returns whether GitHub authentication is available.
pub fn is_authenticated() -> bool {
    github_token().is_some()
}

/// Format a GitHub REST API Authorization header value.
pub fn auth_header(token: &str) -> String {
    format!("Bearer {}", token)
}

/// Build a GitHub REST API URL.
///
/// Substitutes {owner}, {repo}, and {number} path templates.
pub fn build_url(path_template: &str, params: &[(&str, &str)]) -> String {
    let mut url = format!("{}{}", GITHUB_API_BASE, path_template);
    for (key, value) in params {
        url = url.replace(&format!("{{{}}}", key), value);
    }
    url
}

// ─── GitHub Models routing ────────────────────────────────────

/// Returns the GitHub Models endpoint for chat completions.
///
/// GitHub Models is OpenAI-compatible — use the same request format as
/// `GithubCopilotAdapter` in `src/universal/ai_cli.rs`, but point at
/// `GITHUB_MODELS_URL` and use the model ID from `GITHUB_MODELS`.
pub fn models_chat_url() -> &'static str {
    // GitHub Models uses Azure OpenAI endpoint under the hood
    "https://models.inference.ai.azure.com/chat/completions"
}

/// The `uws github models` command lists all available models and their
/// inference endpoint. This constant provides a local fallback when the
/// live catalogue is unreachable.
pub fn local_models_catalogue() -> Vec<String> {
    GITHUB_MODELS
        .iter()
        .map(|m| {
            format!(
                "{{\"id\":\"{}\",\"name\":\"{}\",\"publisher\":\"{}\",\"free\":{}}}",
                m.id, m.display_name, m.publisher, m.free_tier
            )
        })
        .collect()
}

// ─── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_services_have_aliases() {
        for svc in GITHUB_SERVICES {
            assert!(
                !svc.aliases.is_empty(),
                "Service with path {} has no aliases",
                svc.api_path
            );
            // All aliases must start with "github-" or "gh-"
            for alias in svc.aliases {
                assert!(
                    alias.starts_with("github-") || alias.starts_with("gh-"),
                    "Alias '{}' should start with 'github-' or 'gh-'",
                    alias
                );
            }
        }
    }

    #[test]
    fn test_build_url_substitutes_owner_and_repo() {
        let url = build_url(
            "/repos/{owner}/{repo}/issues",
            &[("owner", "acme"), ("repo", "api")],
        );
        assert_eq!(url, "https://api.github.com/repos/acme/api/issues");
    }

    #[test]
    fn test_build_url_leaves_unsubstituted_params() {
        let url = build_url("/repos/{owner}/{repo}/issues", &[("owner", "acme")]);
        // {repo} is not substituted — stays as-is
        assert!(url.contains("{repo}"));
    }

    #[test]
    fn test_auth_header_format() {
        let header = auth_header("ghp_testtoken123");
        assert_eq!(header, "Bearer ghp_testtoken123");
    }

    #[test]
    fn test_github_token_env_var_order() {
        // When both GITHUB_TOKEN and GH_TOKEN are set, GITHUB_TOKEN wins
        std::env::set_var("GITHUB_TOKEN", "primary");
        std::env::set_var("GH_TOKEN", "secondary");
        let token = github_token();
        std::env::remove_var("GITHUB_TOKEN");
        std::env::remove_var("GH_TOKEN");
        assert_eq!(token.as_deref(), Some("primary"));
    }

    #[test]
    fn test_github_token_falls_through_to_gh_token() {
        // Save all three token env vars
        let saved_github = std::env::var("GITHUB_TOKEN").ok();
        let saved_gh = std::env::var("GH_TOKEN").ok();
        let saved_pat = std::env::var("GITHUB_PAT").ok();

        // Remove GITHUB_TOKEN and GITHUB_PAT; only GH_TOKEN should remain
        std::env::remove_var("GITHUB_TOKEN");
        std::env::remove_var("GITHUB_PAT");
        std::env::set_var("GH_TOKEN", "fallback");

        let token = github_token();

        // Restore
        std::env::remove_var("GH_TOKEN");
        if let Some(v) = saved_github { std::env::set_var("GITHUB_TOKEN", v); }
        if let Some(v) = saved_gh    { std::env::set_var("GH_TOKEN", v); }
        if let Some(v) = saved_pat   { std::env::set_var("GITHUB_PAT", v); }

        assert_eq!(token.as_deref(), Some("fallback"));
    }

    #[test]
    fn test_is_authenticated_false_without_env() {
        // Snapshot existing values
        let g = std::env::var("GITHUB_TOKEN").ok();
        let gh = std::env::var("GH_TOKEN").ok();
        let pat = std::env::var("GITHUB_PAT").ok();
        std::env::remove_var("GITHUB_TOKEN");
        std::env::remove_var("GH_TOKEN");
        std::env::remove_var("GITHUB_PAT");

        let result = is_authenticated();

        // Restore
        if let Some(v) = g   { std::env::set_var("GITHUB_TOKEN", v); }
        if let Some(v) = gh  { std::env::set_var("GH_TOKEN", v); }
        if let Some(v) = pat { std::env::set_var("GITHUB_PAT", v); }

        assert!(!result);
    }

    #[test]
    fn test_models_catalogue_not_empty() {
        let catalogue = local_models_catalogue();
        assert!(!catalogue.is_empty());
    }

    #[test]
    fn test_models_catalogue_entries_are_valid_json_fragments() {
        for entry in local_models_catalogue() {
            assert!(entry.starts_with('{'));
            assert!(entry.ends_with('}'));
            assert!(entry.contains("\"id\""));
            assert!(entry.contains("\"publisher\""));
        }
    }

    #[test]
    fn test_github_services_cover_key_use_cases() {
        let paths: Vec<&str> = GITHUB_SERVICES.iter().map(|s| s.api_path).collect();
        // Issues, PRs, Actions, Search, Models must be present
        assert!(paths.iter().any(|p| p.contains("issues")));
        assert!(paths.iter().any(|p| p.contains("pulls")));
        assert!(paths.iter().any(|p| p.contains("actions")));
        assert!(paths.iter().any(|p| p.contains("search")));
        assert!(paths.iter().any(|p| p.contains("models")));
    }

    #[test]
    fn test_constants_are_correct_base_urls() {
        assert_eq!(GITHUB_API_BASE, "https://api.github.com");
        assert!(GITHUB_MODELS_URL.contains("azure.com"));
        assert!(GITHUB_COPILOT_CHAT_URL.contains("githubcopilot.com"));
    }
}
