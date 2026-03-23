// src/prompt_vault.rs
// Aluminum OS — Local Encrypted Prompt Store
//
// Novel Invention #10 — Prompt Vault
//
// AI workflows reuse prompts constantly. The Prompt Vault is a local,
// versioned, searchable store for prompt templates. Each prompt has:
// - A unique slug (e.g., "summarize-email")
// - A version history (FIFO, max N versions kept)
// - Variable placeholders ({{variable}} syntax)
// - Tags for organization
//
// Prompts are stored as plain TOML-like records in ~/.config/uws/prompts/.
// The in-memory representation is pure Rust — no I/O in this module.
//
// Commands:
//   uws prompt save --params '{"slug":"summarize-email"}' --json '{"template":"Summarize: {{text}}"}'
//   uws prompt get --params '{"slug":"summarize-email"}'
//   uws prompt render --params '{"slug":"summarize-email"}' --json '{"text":"Hello world"}'
//   uws prompt list
//   uws prompt delete --params '{"slug":"summarize-email"}'
//
// Author: GitHub Copilot — Aluminum OS Session 2026-03-23

#![allow(dead_code)]

use std::collections::{BTreeMap, VecDeque};

// ─── Constants ────────────────────────────────────────────────────────────

/// Maximum number of versions retained per prompt slug.
pub const MAX_VERSIONS_PER_SLUG: usize = 10;

/// Variable placeholder syntax: {{variable_name}}
pub const PLACEHOLDER_OPEN: &str = "{{";
pub const PLACEHOLDER_CLOSE: &str = "}}";

// ─── Prompt record ────────────────────────────────────────────────────────

/// A single versioned prompt template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptRecord {
    /// Unique identifier (slugified, e.g. "summarize-email").
    pub slug: String,
    /// Human-readable description.
    pub description: String,
    /// The template body. Placeholders use `{{variable}}` syntax.
    pub template: String,
    /// Tags for organization and search.
    pub tags: Vec<String>,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// Version number (1-indexed, increments on each save).
    pub version: u32,
}

impl PromptRecord {
    /// Extract all unique placeholder variable names from the template.
    pub fn variable_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        let mut s = self.template.as_str();
        while let Some(start) = s.find(PLACEHOLDER_OPEN) {
            let after_open = &s[start + PLACEHOLDER_OPEN.len()..];
            if let Some(end) = after_open.find(PLACEHOLDER_CLOSE) {
                let name = after_open[..end].trim().to_string();
                if !name.is_empty() && !names.contains(&name) {
                    names.push(name);
                }
                s = &after_open[end + PLACEHOLDER_CLOSE.len()..];
            } else {
                break;
            }
        }
        names
    }

    /// Render the template by substituting `{{variable}}` placeholders.
    ///
    /// Returns `Err` if any placeholder is missing from `vars`.
    pub fn render(&self, vars: &BTreeMap<String, String>) -> Result<String, PromptVaultError> {
        let mut result = self.template.clone();
        for name in self.variable_names() {
            let placeholder = format!("{PLACEHOLDER_OPEN}{name}{PLACEHOLDER_CLOSE}");
            match vars.get(&name) {
                Some(value) => {
                    result = result.replace(&placeholder, value);
                }
                None => {
                    return Err(PromptVaultError::MissingVariable { name, slug: self.slug.clone() });
                }
            }
        }
        Ok(result)
    }
}

// ─── Vault ────────────────────────────────────────────────────────────────

/// The in-memory Prompt Vault — a keyed store of versioned prompt histories.
#[derive(Debug, Default, Clone)]
pub struct PromptVault {
    /// Maps slug → version history (oldest first).
    prompts: BTreeMap<String, VecDeque<PromptRecord>>,
}

impl PromptVault {
    pub fn new() -> Self {
        Self::default()
    }

    /// Save a new prompt (or a new version of an existing prompt).
    pub fn save(&mut self, record: PromptRecord) {
        let history = self.prompts.entry(record.slug.clone()).or_default();
        history.push_back(record);
        // Trim to max versions
        while history.len() > MAX_VERSIONS_PER_SLUG {
            history.pop_front();
        }
    }

    /// Get the latest version of a prompt by slug.
    pub fn get(&self, slug: &str) -> Option<&PromptRecord> {
        self.prompts.get(slug)?.back()
    }

    /// Get a specific version of a prompt (1-indexed).
    pub fn get_version(&self, slug: &str, version: u32) -> Option<&PromptRecord> {
        self.prompts
            .get(slug)?
            .iter()
            .find(|r| r.version == version)
    }

    /// List all current (latest version) prompts, optionally filtered by tag.
    pub fn list(&self, tag_filter: Option<&str>) -> Vec<&PromptRecord> {
        self.prompts
            .values()
            .filter_map(|history| history.back())
            .filter(|r| match tag_filter {
                Some(tag) => r.tags.iter().any(|t| t == tag),
                None => true,
            })
            .collect()
    }

    /// Search prompts by query string (matches slug, description, template, tags).
    pub fn search(&self, query: &str) -> Vec<&PromptRecord> {
        let q = query.to_lowercase();
        self.list(None)
            .into_iter()
            .filter(|r| {
                r.slug.to_lowercase().contains(&q)
                    || r.description.to_lowercase().contains(&q)
                    || r.template.to_lowercase().contains(&q)
                    || r.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect()
    }

    /// Delete all versions of a prompt.
    pub fn delete(&mut self, slug: &str) -> bool {
        self.prompts.remove(slug).is_some()
    }

    /// Get version history for a slug.
    pub fn history(&self, slug: &str) -> Vec<&PromptRecord> {
        self.prompts
            .get(slug)
            .map(|h| h.iter().collect())
            .unwrap_or_default()
    }

    /// Total number of unique prompt slugs in the vault.
    pub fn len(&self) -> usize {
        self.prompts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.prompts.is_empty()
    }
}

// ─── Slug validation ──────────────────────────────────────────────────────

/// Validate a prompt slug: lowercase alphanumeric + hyphens, 2–64 chars.
pub fn validate_slug(slug: &str) -> Result<(), PromptVaultError> {
    if slug.len() < 2 || slug.len() > 64 {
        return Err(PromptVaultError::InvalidSlug {
            reason: format!("Slug must be 2–64 characters, got {}", slug.len()),
        });
    }
    if !slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(PromptVaultError::InvalidSlug {
            reason: "Slug must contain only lowercase alphanumeric characters and hyphens".to_string(),
        });
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(PromptVaultError::InvalidSlug {
            reason: "Slug must not start or end with a hyphen".to_string(),
        });
    }
    Ok(())
}

// ─── Error type ──────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum PromptVaultError {
    NotFound { slug: String },
    InvalidSlug { reason: String },
    MissingVariable { name: String, slug: String },
}

impl std::fmt::Display for PromptVaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromptVaultError::NotFound { slug } => write!(f, "Prompt not found: '{slug}'"),
            PromptVaultError::InvalidSlug { reason } => write!(f, "Invalid slug: {reason}"),
            PromptVaultError::MissingVariable { name, slug } => {
                write!(f, "Missing variable '{name}' required by prompt '{slug}'")
            }
        }
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_prompt(slug: &str, template: &str) -> PromptRecord {
        PromptRecord {
            slug: slug.to_string(),
            description: format!("Test prompt: {slug}"),
            template: template.to_string(),
            tags: vec!["test".to_string()],
            created_at: "2026-01-01T00:00:00Z".to_string(),
            version: 1,
        }
    }

    #[test]
    fn test_variable_names_basic() {
        let p = make_prompt("test", "Hello {{name}}, you have {{count}} messages.");
        let vars = p.variable_names();
        assert_eq!(vars, vec!["name", "count"]);
    }

    #[test]
    fn test_variable_names_empty() {
        let p = make_prompt("test", "No variables here.");
        assert!(p.variable_names().is_empty());
    }

    #[test]
    fn test_variable_names_deduplicates() {
        let p = make_prompt("test", "{{x}} and {{x}} again.");
        assert_eq!(p.variable_names(), vec!["x"]);
    }

    #[test]
    fn test_render_basic() {
        let p = make_prompt("test", "Hello {{name}}!");
        let mut vars = BTreeMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        assert_eq!(p.render(&vars).unwrap(), "Hello Alice!");
    }

    #[test]
    fn test_render_multiple_vars() {
        let p = make_prompt("test", "{{greeting}}, {{name}}. You have {{n}} new emails.");
        let mut vars = BTreeMap::new();
        vars.insert("greeting".to_string(), "Good morning".to_string());
        vars.insert("name".to_string(), "Bob".to_string());
        vars.insert("n".to_string(), "5".to_string());
        assert_eq!(p.render(&vars).unwrap(), "Good morning, Bob. You have 5 new emails.");
    }

    #[test]
    fn test_render_missing_variable_returns_error() {
        let p = make_prompt("test", "Hello {{name}}!");
        let vars = BTreeMap::new();
        let err = p.render(&vars).unwrap_err();
        assert!(matches!(err, PromptVaultError::MissingVariable { .. }));
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn test_vault_save_and_get() {
        let mut vault = PromptVault::new();
        vault.save(make_prompt("greet", "Hello {{name}}!"));
        assert!(vault.get("greet").is_some());
        assert_eq!(vault.get("greet").unwrap().slug, "greet");
    }

    #[test]
    fn test_vault_get_unknown_returns_none() {
        let vault = PromptVault::new();
        assert!(vault.get("nonexistent").is_none());
    }

    #[test]
    fn test_vault_versioning() {
        let mut vault = PromptVault::new();
        vault.save(PromptRecord { version: 1, ..make_prompt("p", "v1") });
        vault.save(PromptRecord { version: 2, ..make_prompt("p", "v2") });
        assert_eq!(vault.get("p").unwrap().version, 2);
        assert_eq!(vault.history("p").len(), 2);
    }

    #[test]
    fn test_vault_max_versions_enforced() {
        let mut vault = PromptVault::new();
        for i in 1..=(MAX_VERSIONS_PER_SLUG + 5) as u32 {
            vault.save(PromptRecord { version: i, ..make_prompt("p", &format!("v{i}")) });
        }
        assert!(vault.history("p").len() <= MAX_VERSIONS_PER_SLUG);
    }

    #[test]
    fn test_vault_list_all() {
        let mut vault = PromptVault::new();
        vault.save(make_prompt("a", "template a"));
        vault.save(make_prompt("b", "template b"));
        assert_eq!(vault.list(None).len(), 2);
    }

    #[test]
    fn test_vault_list_by_tag() {
        let mut vault = PromptVault::new();
        vault.save(PromptRecord {
            tags: vec!["email".to_string()],
            ..make_prompt("email-summary", "summarize email")
        });
        vault.save(PromptRecord {
            tags: vec!["code".to_string()],
            ..make_prompt("code-review", "review code")
        });
        let email_prompts = vault.list(Some("email"));
        assert_eq!(email_prompts.len(), 1);
        assert_eq!(email_prompts[0].slug, "email-summary");
    }

    #[test]
    fn test_vault_search() {
        let mut vault = PromptVault::new();
        vault.save(make_prompt("email-summary", "Summarize this email thread"));
        vault.save(make_prompt("code-review", "Review this code for bugs"));
        let results = vault.search("email");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "email-summary");
    }

    #[test]
    fn test_vault_delete() {
        let mut vault = PromptVault::new();
        vault.save(make_prompt("to-delete", "template"));
        assert!(vault.delete("to-delete"));
        assert!(vault.get("to-delete").is_none());
        assert!(!vault.delete("to-delete")); // already deleted
    }

    #[test]
    fn test_validate_slug_valid() {
        assert!(validate_slug("my-prompt").is_ok());
        assert!(validate_slug("summarize-email-v2").is_ok());
        assert!(validate_slug("ab").is_ok());
    }

    #[test]
    fn test_validate_slug_too_short() {
        assert!(validate_slug("a").is_err());
    }

    #[test]
    fn test_validate_slug_invalid_chars() {
        assert!(validate_slug("has spaces").is_err());
        assert!(validate_slug("Has_Underscores").is_err());
    }

    #[test]
    fn test_validate_slug_starts_or_ends_with_hyphen() {
        assert!(validate_slug("-bad").is_err());
        assert!(validate_slug("bad-").is_err());
    }

    #[test]
    fn test_error_display() {
        let e = PromptVaultError::NotFound { slug: "foo".to_string() };
        assert!(e.to_string().contains("foo"));
        let e2 = PromptVaultError::MissingVariable {
            name: "email".to_string(),
            slug: "my-prompt".to_string(),
        };
        assert!(e2.to_string().contains("email"));
        assert!(e2.to_string().contains("my-prompt"));
    }
}
