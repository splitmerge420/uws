// src/universal_io.rs
// Aluminum OS — Universal I/O Abstraction Layer
//
// This module strips proprietary SaaS formatting from documents retrieved
// via Google Workspace, Microsoft 365, or any other provider, converting
// them into a locally-owned `UniversalDocument` (standard Markdown body +
// JSON-compatible metadata frontmatter).
//
// Design principles:
//   - Provider neutrality: Google and Microsoft become "dumb pipes".
//   - Local sovereignty: the user's machine is the source of truth.
//   - Interoperability: any document can move between ecosystems freely.
//
// Invariants Enforced: INV-1 (Sovereignty), INV-6 (Provider Abstraction)

#![allow(dead_code)]

use std::collections::BTreeMap;

// ─── Universal Document ───────────────────────────────────────────────────────

/// A provider-agnostic document representation.
///
/// `UniversalDocument` is the canonical, unshackled form of any document
/// pulled from a proprietary SaaS platform. The body is standard Markdown;
/// metadata is a flat, JSON-compatible key/value map (BTreeMap for stable
/// serialisation order).
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct UniversalDocument {
    /// Unique identifier (provider-assigned or locally generated).
    pub id: String,
    /// Human-readable document title.
    pub title: String,
    /// Document body rendered as standard Markdown.
    pub body_markdown: String,
    /// Flat metadata / frontmatter (e.g. author, created_at, source_url).
    pub metadata: BTreeMap<String, String>,
}

impl UniversalDocument {
    /// Create a new `UniversalDocument` with empty metadata.
    pub fn new(id: impl Into<String>, title: impl Into<String>, body_markdown: impl Into<String>) -> Self {
        UniversalDocument {
            id: id.into(),
            title: title.into(),
            body_markdown: body_markdown.into(),
            metadata: BTreeMap::new(),
        }
    }

    /// Attach a metadata key/value pair.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

// ─── SaaS Connector Trait ────────────────────────────────────────────────────

/// Abstraction over any proprietary SaaS data source.
///
/// Implementors are responsible for:
/// 1. Authenticating with the upstream provider.
/// 2. Fetching the raw, proprietary document representation.
/// 3. Stripping provider-specific formatting and metadata noise.
/// 4. Returning a clean `UniversalDocument` that the local OS owns.
///
/// The trait is intentionally synchronous in its stub form. When the
/// `tokio`/`reqwest` dependencies are enabled (Phase 2 in `Cargo.toml`),
/// implementations should use async versions of these methods.
pub trait SaaSConnector {
    /// The human-readable name of the upstream provider (e.g. `"Google Workspace"`).
    fn provider_name(&self) -> &str;

    /// Pull a single document by `document_id` from the upstream provider,
    /// strip all proprietary formatting, and return a `UniversalDocument`.
    ///
    /// Returns `Err` with a descriptive message on authentication failure,
    /// network error, or unsupported document type.
    fn pull_and_strip(&self, document_id: &str) -> Result<UniversalDocument, String>;

    /// List the IDs of documents available in the upstream source.
    ///
    /// Returns `Err` with a descriptive message on failure.
    fn list_document_ids(&self) -> Result<Vec<String>, String>;
}

// ─── Google Workspace Stub ───────────────────────────────────────────────────

/// Stub implementation of `SaaSConnector` for Google Workspace.
///
/// This is a mock/stub that demonstrates the intended interface without
/// making real network calls. Replace the body of `pull_and_strip` and
/// `list_document_ids` with real Google Docs / Drive API calls once the
/// `reqwest` + `tokio` dependencies are enabled in `Cargo.toml`.
pub struct GoogleWorkspaceConnector {
    /// OAuth2 access token (or a placeholder value for the stub).
    pub access_token: String,
}

impl GoogleWorkspaceConnector {
    /// Create a new connector with the given OAuth2 access token.
    pub fn new(access_token: impl Into<String>) -> Self {
        GoogleWorkspaceConnector {
            access_token: access_token.into(),
        }
    }

    /// Simulate stripping Google Docs JSON content into Markdown.
    ///
    /// A real implementation would call the Google Docs REST API
    /// (`GET https://docs.googleapis.com/v1/documents/{documentId}`)
    /// and walk the `StructuralElement` tree to emit Markdown.
    fn strip_gdoc_to_markdown(raw_title: &str, raw_body: &str) -> String {
        // Stub: return a Markdown representation of the title + body.
        format!("# {raw_title}\n\n{raw_body}")
    }
}

impl SaaSConnector for GoogleWorkspaceConnector {
    fn provider_name(&self) -> &str {
        "Google Workspace"
    }

    fn pull_and_strip(&self, document_id: &str) -> Result<UniversalDocument, String> {
        if self.access_token.is_empty() {
            return Err(String::from("Google Workspace: access_token is empty"));
        }

        // ── Stub ─────────────────────────────────────────────────────────
        // In a real implementation this would:
        //   1. GET https://docs.googleapis.com/v1/documents/{document_id}
        //   2. Parse the structural elements tree.
        //   3. Convert paragraphs → Markdown headings / paragraphs / lists.
        //   4. Extract named styles, inline objects, etc.
        // For now we return a synthesised placeholder document.
        let title = format!("Google Doc {document_id}");
        let raw_body = "This document was retrieved from Google Workspace and stripped of proprietary formatting.";
        let body_md = Self::strip_gdoc_to_markdown(&title, raw_body);

        let doc = UniversalDocument::new(document_id, &title, body_md)
            .with_metadata("source", "Google Workspace")
            .with_metadata("source_url", format!("https://docs.google.com/document/d/{document_id}/edit"))
            .with_metadata("connector_version", "stub-0.1");

        Ok(doc)
    }

    fn list_document_ids(&self) -> Result<Vec<String>, String> {
        if self.access_token.is_empty() {
            return Err(String::from("Google Workspace: access_token is empty"));
        }

        // Stub: return a fixed list of placeholder document IDs.
        // A real implementation would call:
        //   GET https://www.googleapis.com/drive/v3/files?q=mimeType='application/vnd.google-apps.document'
        Ok(vec![
            String::from("stub-doc-id-001"),
            String::from("stub-doc-id-002"),
            String::from("stub-doc-id-003"),
        ])
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── UniversalDocument ──────────────────────────────────────────────────

    #[test]
    fn test_universal_document_new() {
        let doc = UniversalDocument::new("id-1", "Hello World", "# Hello\n\nWorld");
        assert_eq!(doc.id, "id-1");
        assert_eq!(doc.title, "Hello World");
        assert_eq!(doc.body_markdown, "# Hello\n\nWorld");
        assert!(doc.metadata.is_empty());
    }

    #[test]
    fn test_universal_document_with_metadata() {
        let doc = UniversalDocument::new("id-2", "Test", "body")
            .with_metadata("author", "Dave")
            .with_metadata("source", "Google Workspace");
        assert_eq!(doc.metadata.get("author").map(String::as_str), Some("Dave"));
        assert_eq!(doc.metadata.get("source").map(String::as_str), Some("Google Workspace"));
    }

    // ── GoogleWorkspaceConnector ───────────────────────────────────────────

    #[test]
    fn test_provider_name() {
        let connector = GoogleWorkspaceConnector::new("token");
        assert_eq!(connector.provider_name(), "Google Workspace");
    }

    #[test]
    fn test_pull_and_strip_returns_document() {
        let connector = GoogleWorkspaceConnector::new("fake-token");
        let result = connector.pull_and_strip("doc-abc");
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.id, "doc-abc");
        assert!(doc.body_markdown.contains("doc-abc"));
        assert_eq!(doc.metadata.get("source").map(String::as_str), Some("Google Workspace"));
    }

    #[test]
    fn test_pull_and_strip_empty_token_returns_err() {
        let connector = GoogleWorkspaceConnector::new("");
        let result = connector.pull_and_strip("doc-xyz");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("access_token is empty"));
    }

    #[test]
    fn test_list_document_ids_returns_stubs() {
        let connector = GoogleWorkspaceConnector::new("fake-token");
        let result = connector.list_document_ids();
        assert!(result.is_ok());
        let ids = result.unwrap();
        assert!(!ids.is_empty());
    }

    #[test]
    fn test_list_document_ids_empty_token_returns_err() {
        let connector = GoogleWorkspaceConnector::new("");
        let result = connector.list_document_ids();
        assert!(result.is_err());
    }

    #[test]
    fn test_strip_gdoc_to_markdown_format() {
        let md = GoogleWorkspaceConnector::strip_gdoc_to_markdown("My Doc", "Some content.");
        assert!(md.starts_with("# My Doc"));
        assert!(md.contains("Some content."));
    }
}
