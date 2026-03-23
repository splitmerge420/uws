// src/persona_manager.rs
// Aluminum OS — Multi-Identity Persona Manager
//
// Novel Invention #15 — Persona Manager
//
// Modern professionals wear multiple hats: work identity, personal identity,
// contractor/freelance identity, family identity. Each identity has different
// provider credentials, different default configurations, and different data
// domains.
//
// The Persona Manager lets users switch between named profiles (personas)
// in a single command. Each persona carries:
// - A set of environment variable overrides (credentials per provider)
// - A default configuration (timezone, output format, etc.)
// - A set of "active providers" (e.g., your personal persona might not have
//   Slack or Linear configured)
// - Metadata (display name, color, emoji)
//
// Commands:
//   uws persona list
//   uws persona switch --params '{"name":"work"}'
//   uws persona current
//   uws persona create --params '{"name":"freelance"}' --json '{"providers":["github","stripe"]}'
//   uws persona delete --params '{"name":"freelance"}'
//
// Author: GitHub Copilot — Aluminum OS Session 2026-03-23

#![allow(dead_code)]

use std::collections::BTreeMap;

// ─── Persona definition ───────────────────────────────────────────────────

/// A named identity profile with provider-specific credentials and settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Persona {
    /// Unique slug (e.g., "work", "personal", "freelance").
    pub name: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Emoji icon for the persona.
    pub icon: String,
    /// Environment variable overrides (credential keys → values).
    /// These are applied when the persona is active.
    pub env_overrides: BTreeMap<String, String>,
    /// Providers enabled for this persona.
    pub active_providers: Vec<String>,
    /// Default output format for this persona.
    pub default_format: OutputFormat,
    /// Default timezone (e.g., "America/New_York").
    pub timezone: String,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// Whether this is the default persona (auto-activated on startup).
    pub is_default: bool,
}

impl Persona {
    pub fn new(name: impl Into<String>, display_name: impl Into<String>) -> Self {
        Persona {
            name: name.into(),
            display_name: display_name.into(),
            icon: "👤".to_string(),
            env_overrides: BTreeMap::new(),
            active_providers: Vec::new(),
            default_format: OutputFormat::Json,
            timezone: "UTC".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            is_default: false,
        }
    }

    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.active_providers.push(provider.into());
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_overrides.insert(key.into(), value.into());
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    pub fn with_timezone(mut self, tz: impl Into<String>) -> Self {
        self.timezone = tz.into();
        self
    }

    /// Check if a specific provider is enabled for this persona.
    pub fn has_provider(&self, provider: &str) -> bool {
        self.active_providers.iter().any(|p| p == provider)
    }

    /// Build the effective environment variable map for this persona.
    /// These would be applied when switching to this persona.
    pub fn effective_env(&self) -> BTreeMap<String, String> {
        self.env_overrides.clone()
    }
}

// ─── Output format ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Table,
    Yaml,
    Csv,
}

impl OutputFormat {
    pub fn as_str(&self) -> &str {
        match self {
            OutputFormat::Json => "json",
            OutputFormat::Table => "table",
            OutputFormat::Yaml => "yaml",
            OutputFormat::Csv => "csv",
        }
    }

    pub fn parse(s: &str) -> OutputFormat {
        match s {
            "table" => OutputFormat::Table,
            "yaml" => OutputFormat::Yaml,
            "csv" => OutputFormat::Csv,
            _ => OutputFormat::Json,
        }
    }
}

// ─── Persona registry ─────────────────────────────────────────────────────

/// The in-memory registry of all personas.
#[derive(Debug, Default, Clone)]
pub struct PersonaRegistry {
    personas: BTreeMap<String, Persona>,
    active: Option<String>,
}

impl PersonaRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new persona.
    pub fn register(&mut self, persona: Persona) -> Result<(), PersonaError> {
        validate_persona_name(&persona.name)?;
        if self.personas.contains_key(&persona.name) {
            return Err(PersonaError::AlreadyExists(persona.name.clone()));
        }
        // If this is the first or marked as default, set it as active
        if self.personas.is_empty() || persona.is_default {
            self.active = Some(persona.name.clone());
        }
        self.personas.insert(persona.name.clone(), persona);
        Ok(())
    }

    /// Update an existing persona.
    pub fn update(&mut self, persona: Persona) -> Result<(), PersonaError> {
        if !self.personas.contains_key(&persona.name) {
            return Err(PersonaError::NotFound(persona.name.clone()));
        }
        self.personas.insert(persona.name.clone(), persona);
        Ok(())
    }

    /// Switch the active persona.
    pub fn switch(&mut self, name: &str) -> Result<&Persona, PersonaError> {
        if !self.personas.contains_key(name) {
            return Err(PersonaError::NotFound(name.to_string()));
        }
        self.active = Some(name.to_string());
        Ok(&self.personas[name])
    }

    /// Get the currently active persona.
    pub fn current(&self) -> Option<&Persona> {
        self.active.as_deref().and_then(|name| self.personas.get(name))
    }

    /// Get a persona by name.
    pub fn get(&self, name: &str) -> Option<&Persona> {
        self.personas.get(name)
    }

    /// List all registered personas.
    pub fn list(&self) -> Vec<&Persona> {
        self.personas.values().collect()
    }

    /// Delete a persona.
    pub fn delete(&mut self, name: &str) -> Result<(), PersonaError> {
        if !self.personas.contains_key(name) {
            return Err(PersonaError::NotFound(name.to_string()));
        }
        // Cannot delete the active persona
        if self.active.as_deref() == Some(name) {
            return Err(PersonaError::CannotDeleteActive(name.to_string()));
        }
        self.personas.remove(name);
        Ok(())
    }

    /// Get the environment variable overrides for the active persona.
    pub fn active_env(&self) -> BTreeMap<String, String> {
        self.current()
            .map(|p| p.effective_env())
            .unwrap_or_default()
    }

    /// Check if a provider is enabled in the active persona.
    pub fn is_provider_active(&self, provider: &str) -> bool {
        self.current()
            .map(|p| p.has_provider(provider))
            .unwrap_or(true) // If no persona is active, all providers are enabled
    }

    pub fn len(&self) -> usize {
        self.personas.len()
    }

    pub fn is_empty(&self) -> bool {
        self.personas.is_empty()
    }
}

// ─── Name validation ──────────────────────────────────────────────────────

pub fn validate_persona_name(name: &str) -> Result<(), PersonaError> {
    if name.is_empty() || name.len() > 32 {
        return Err(PersonaError::InvalidName(
            format!("Persona name must be 1–32 characters, got {}", name.len())
        ));
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(PersonaError::InvalidName(
            "Persona name must contain only alphanumeric characters, hyphens, or underscores".to_string()
        ));
    }
    Ok(())
}

// ─── Error type ──────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum PersonaError {
    NotFound(String),
    AlreadyExists(String),
    CannotDeleteActive(String),
    InvalidName(String),
}

impl std::fmt::Display for PersonaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersonaError::NotFound(name) => write!(f, "Persona not found: '{name}'"),
            PersonaError::AlreadyExists(name) => write!(f, "Persona already exists: '{name}'"),
            PersonaError::CannotDeleteActive(name) => {
                write!(f, "Cannot delete active persona '{name}'. Switch first.")
            }
            PersonaError::InvalidName(msg) => write!(f, "Invalid persona name: {msg}"),
        }
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn work_persona() -> Persona {
        Persona::new("work", "Work Profile")
            .with_provider("github")
            .with_provider("slack")
            .with_provider("linear")
            .with_env("GITHUB_TOKEN", "ghp_work_token")
            .with_icon("💼")
            .with_timezone("America/New_York")
    }

    fn personal_persona() -> Persona {
        Persona::new("personal", "Personal Profile")
            .with_provider("github")
            .with_provider("gmail")
            .with_env("GITHUB_TOKEN", "ghp_personal_token")
            .with_icon("🏠")
    }

    #[test]
    fn test_persona_has_provider() {
        let p = work_persona();
        assert!(p.has_provider("github"));
        assert!(p.has_provider("slack"));
        assert!(!p.has_provider("stripe"));
    }

    #[test]
    fn test_persona_effective_env() {
        let p = work_persona();
        let env = p.effective_env();
        assert_eq!(env.get("GITHUB_TOKEN").map(|s| s.as_str()), Some("ghp_work_token"));
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut reg = PersonaRegistry::new();
        reg.register(work_persona()).unwrap();
        assert!(reg.get("work").is_some());
    }

    #[test]
    fn test_registry_first_persona_becomes_active() {
        let mut reg = PersonaRegistry::new();
        reg.register(work_persona()).unwrap();
        assert_eq!(reg.current().map(|p| p.name.as_str()), Some("work"));
    }

    #[test]
    fn test_registry_switch_persona() {
        let mut reg = PersonaRegistry::new();
        reg.register(work_persona()).unwrap();
        reg.register(personal_persona()).unwrap();
        reg.switch("personal").unwrap();
        assert_eq!(reg.current().map(|p| p.name.as_str()), Some("personal"));
    }

    #[test]
    fn test_registry_switch_unknown_returns_error() {
        let mut reg = PersonaRegistry::new();
        reg.register(work_persona()).unwrap();
        assert!(reg.switch("nonexistent").is_err());
    }

    #[test]
    fn test_registry_duplicate_registration_returns_error() {
        let mut reg = PersonaRegistry::new();
        reg.register(work_persona()).unwrap();
        let err = reg.register(work_persona()).unwrap_err();
        assert!(matches!(err, PersonaError::AlreadyExists(_)));
    }

    #[test]
    fn test_registry_delete_inactive_persona() {
        let mut reg = PersonaRegistry::new();
        reg.register(work_persona()).unwrap();
        reg.register(personal_persona()).unwrap();
        // work is active (first registered); delete personal
        reg.delete("personal").unwrap();
        assert!(reg.get("personal").is_none());
    }

    #[test]
    fn test_registry_cannot_delete_active_persona() {
        let mut reg = PersonaRegistry::new();
        reg.register(work_persona()).unwrap();
        let err = reg.delete("work").unwrap_err();
        assert!(matches!(err, PersonaError::CannotDeleteActive(_)));
    }

    #[test]
    fn test_active_env_returns_active_persona_env() {
        let mut reg = PersonaRegistry::new();
        reg.register(work_persona()).unwrap();
        let env = reg.active_env();
        assert_eq!(env.get("GITHUB_TOKEN").map(|s| s.as_str()), Some("ghp_work_token"));
    }

    #[test]
    fn test_is_provider_active_for_active_persona() {
        let mut reg = PersonaRegistry::new();
        reg.register(work_persona()).unwrap();
        assert!(reg.is_provider_active("github"));
        assert!(!reg.is_provider_active("stripe"));
    }

    #[test]
    fn test_list_all_personas() {
        let mut reg = PersonaRegistry::new();
        reg.register(work_persona()).unwrap();
        reg.register(personal_persona()).unwrap();
        assert_eq!(reg.list().len(), 2);
    }

    #[test]
    fn test_output_format_roundtrip() {
        assert_eq!(OutputFormat::parse("table"), OutputFormat::Table);
        assert_eq!(OutputFormat::parse("yaml"), OutputFormat::Yaml);
        assert_eq!(OutputFormat::parse("json"), OutputFormat::Json);
        assert_eq!(OutputFormat::parse("unknown"), OutputFormat::Json);
    }

    #[test]
    fn test_validate_persona_name_valid() {
        assert!(validate_persona_name("work").is_ok());
        assert!(validate_persona_name("my-work_profile").is_ok());
    }

    #[test]
    fn test_validate_persona_name_invalid_chars() {
        assert!(validate_persona_name("has spaces").is_err());
        assert!(validate_persona_name("has.dots").is_err());
    }

    #[test]
    fn test_error_display() {
        let e = PersonaError::NotFound("xyz".to_string());
        assert!(e.to_string().contains("xyz"));
        let e2 = PersonaError::CannotDeleteActive("work".to_string());
        assert!(e2.to_string().contains("Switch first"));
    }
}
