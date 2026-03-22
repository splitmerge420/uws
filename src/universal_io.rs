// src/universal_io.rs
// Aluminum OS — Universal I/O Abstraction Layer
//
// This module strips proprietary SaaS formatting from documents retrieved
// via Google Workspace, Microsoft 365, Apple Notes, or plain-text sources,
// converting them into a locally-owned `UniversalDocument` (standard Markdown
// body + JSON-compatible metadata frontmatter).  The reverse operation
// (`repatriate`) converts a `UniversalDocument` back into the provider's
// native format, enabling true bi-directional portability.
//
// Design principles:
//   - Provider neutrality: every SaaS platform is a "dumb pipe".
//   - Local sovereignty: the user's machine is the source of truth.
//   - Interoperability: any document can move between ecosystems freely.
//   - Bi-directionality: extract → edit locally → repatriate, zero lock-in.
//
// Invariants Enforced: INV-1 (Sovereignty), INV-6 (Provider Abstraction)

#![allow(dead_code)]

use std::collections::BTreeMap;

// ─── Provider Format ─────────────────────────────────────────────────────────

/// The native document format of an upstream SaaS provider.
///
/// Used by `SaaSConnector::repatriate` to know which wire format to produce
/// when writing a `UniversalDocument` back to the provider.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderFormat {
    /// Google Docs (application/vnd.google-apps.document) — structural JSON
    GoogleDoc,
    /// Microsoft Word (.docx) — OOXML
    MicrosoftWord,
    /// Apple Notes — proprietary XHTML stored in CloudKit
    AppleNote,
    /// Plain text / Markdown — no conversion needed
    PlainText,
}

impl ProviderFormat {
    /// Return the MIME type associated with this format.
    pub fn mime_type(&self) -> &str {
        match self {
            ProviderFormat::GoogleDoc => "application/vnd.google-apps.document",
            ProviderFormat::MicrosoftWord => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ProviderFormat::AppleNote => "text/html",
            ProviderFormat::PlainText => "text/plain",
        }
    }
}

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
/// The `repatriate` method closes the loop — it converts a `UniversalDocument`
/// back into the provider's native wire format so it can be written back.
///
/// The trait is intentionally synchronous in its stub form. When the
/// `tokio`/`reqwest` dependencies are enabled (Phase 2 in `Cargo.toml`),
/// implementations should use async versions of these methods.
pub trait SaaSConnector {
    /// The human-readable name of the upstream provider (e.g. `"Google Workspace"`).
    fn provider_name(&self) -> &str;

    /// The native document format this connector produces and consumes.
    fn provider_format(&self) -> ProviderFormat;

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

    /// Convert a `UniversalDocument` back into the provider's native wire
    /// format (the reverse of `pull_and_strip`).
    ///
    /// The returned `String` is the serialised representation ready to be
    /// written to the provider's API (e.g. OOXML for Word, XHTML for Apple
    /// Notes, Google Docs JSON for Google Docs).  For `PlainText` this is
    /// simply the Markdown body.
    ///
    /// Returns `Err` with a descriptive message if the document cannot be
    /// repatriated to this provider's format.
    fn repatriate(&self, doc: &UniversalDocument) -> Result<String, String>;
}

// ─── Google Workspace Connector ──────────────────────────────────────────────

/// Stub implementation of `SaaSConnector` for Google Workspace (Google Docs).
///
/// Replace the stub bodies with real Google Docs REST API calls once the
/// `reqwest` + `tokio` dependencies are enabled in `Cargo.toml`.
pub struct GoogleWorkspaceConnector {
    /// OAuth2 access token (or a placeholder value for the stub).
    pub access_token: String,
}

impl GoogleWorkspaceConnector {
    /// Create a new connector with the given OAuth2 access token.
    pub fn new(access_token: impl Into<String>) -> Self {
        GoogleWorkspaceConnector { access_token: access_token.into() }
    }

    fn check_token(&self, provider: &str) -> Result<(), String> {
        if self.access_token.is_empty() {
            Err(format!("{provider}: access_token is empty"))
        } else {
            Ok(())
        }
    }

    /// Simulate stripping Google Docs JSON content into Markdown.
    ///
    /// Real implementation: call `GET https://docs.googleapis.com/v1/documents/{id}`
    /// and walk the `StructuralElement` tree to emit Markdown.
    fn strip_gdoc_to_markdown(raw_title: &str, raw_body: &str) -> String {
        format!("# {raw_title}\n\n{raw_body}")
    }

    /// Simulate converting Markdown back into a minimal Google Docs JSON patch body.
    ///
    /// Real implementation: build a `BatchUpdateDocumentRequest` with
    /// `InsertTextRequest` / `UpdateParagraphStyleRequest` elements.
    fn markdown_to_gdoc_json(doc: &UniversalDocument) -> String {
        // Stub: produce a minimal JSON representation that a real implementation
        // would expand into a full BatchUpdateDocumentRequest.
        format!(
            r#"{{"title":"{title}","body":{{"content":[{{"paragraph":{{"elements":[{{"textRun":{{"content":"{body}"}}}}]}}}}]}}}}"#,
            title = doc.title.replace('"', r#"\""#),
            body = doc.body_markdown.replace('"', r#"\""#).replace('\n', "\\n"),
        )
    }
}

impl SaaSConnector for GoogleWorkspaceConnector {
    fn provider_name(&self) -> &str {
        "Google Workspace"
    }

    fn provider_format(&self) -> ProviderFormat {
        ProviderFormat::GoogleDoc
    }

    fn pull_and_strip(&self, document_id: &str) -> Result<UniversalDocument, String> {
        self.check_token("Google Workspace")?;

        // Stub — real impl: GET https://docs.googleapis.com/v1/documents/{document_id}
        let title = format!("Google Doc {document_id}");
        let raw_body = "This document was retrieved from Google Workspace and stripped of proprietary formatting.";
        let body_md = Self::strip_gdoc_to_markdown(&title, raw_body);

        Ok(UniversalDocument::new(document_id, &title, body_md)
            .with_metadata("source", "Google Workspace")
            .with_metadata("source_url", format!("https://docs.google.com/document/d/{document_id}/edit"))
            .with_metadata("connector_version", "stub-0.1"))
    }

    fn list_document_ids(&self) -> Result<Vec<String>, String> {
        self.check_token("Google Workspace")?;

        // Stub — real impl: GET https://www.googleapis.com/drive/v3/files
        //   ?q=mimeType='application/vnd.google-apps.document'
        Ok(vec![
            String::from("stub-gdoc-id-001"),
            String::from("stub-gdoc-id-002"),
            String::from("stub-gdoc-id-003"),
        ])
    }

    fn repatriate(&self, doc: &UniversalDocument) -> Result<String, String> {
        self.check_token("Google Workspace")?;

        // Stub — real impl: POST https://docs.googleapis.com/v1/documents/{id}:batchUpdate
        Ok(Self::markdown_to_gdoc_json(doc))
    }
}

// ─── Microsoft Word Connector ────────────────────────────────────────────────

/// Stub implementation of `SaaSConnector` for Microsoft 365 (Word / OneDrive).
///
/// Replace the stub bodies with real Microsoft Graph API calls once the
/// `reqwest` + `tokio` dependencies are enabled in `Cargo.toml`.
pub struct MicrosoftWordConnector {
    /// Microsoft Graph OAuth2 access token.
    pub access_token: String,
}

impl MicrosoftWordConnector {
    /// Create a new connector with the given Microsoft Graph OAuth2 access token.
    pub fn new(access_token: impl Into<String>) -> Self {
        MicrosoftWordConnector { access_token: access_token.into() }
    }

    fn check_token(&self) -> Result<(), String> {
        if self.access_token.is_empty() {
            Err(String::from("Microsoft Word: access_token is empty"))
        } else {
            Ok(())
        }
    }

    /// Simulate stripping OOXML (Word .docx) content into Markdown.
    ///
    /// Real implementation: unzip the .docx, parse `word/document.xml`,
    /// walk `<w:p>` / `<w:r>` elements and convert to Markdown.
    fn strip_docx_to_markdown(raw_title: &str, raw_body: &str) -> String {
        format!("# {raw_title}\n\n{raw_body}")
    }

    /// Simulate converting Markdown back into a minimal OOXML document body.
    ///
    /// Real implementation: produce a valid `word/document.xml` fragment
    /// inside a .docx ZIP structure via the Open XML SDK conventions.
    fn markdown_to_ooxml(doc: &UniversalDocument) -> String {
        // Stub: produce a minimal OOXML-like representation.
        format!(
            r#"<?xml version="1.0"?><w:document><w:body><w:p><w:r><w:t>{title}</w:t></w:r></w:p><w:p><w:r><w:t>{body}</w:t></w:r></w:p></w:body></w:document>"#,
            title = doc.title,
            body = doc.body_markdown,
        )
    }
}

impl SaaSConnector for MicrosoftWordConnector {
    fn provider_name(&self) -> &str {
        "Microsoft Word"
    }

    fn provider_format(&self) -> ProviderFormat {
        ProviderFormat::MicrosoftWord
    }

    fn pull_and_strip(&self, document_id: &str) -> Result<UniversalDocument, String> {
        self.check_token()?;

        // Stub — real impl: GET https://graph.microsoft.com/v1.0/me/drive/items/{document_id}/content
        let title = format!("Word Document {document_id}");
        let raw_body = "This document was retrieved from Microsoft Word and stripped of OOXML formatting.";
        let body_md = Self::strip_docx_to_markdown(&title, raw_body);

        Ok(UniversalDocument::new(document_id, &title, body_md)
            .with_metadata("source", "Microsoft Word")
            .with_metadata("source_url", format!("https://onedrive.live.com/view.aspx?id={document_id}"))
            .with_metadata("connector_version", "stub-0.1"))
    }

    fn list_document_ids(&self) -> Result<Vec<String>, String> {
        self.check_token()?;

        // Stub — real impl: GET https://graph.microsoft.com/v1.0/me/drive/root/children
        //   ?$filter=file/mimeType eq 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'
        Ok(vec![
            String::from("stub-word-id-001"),
            String::from("stub-word-id-002"),
        ])
    }

    fn repatriate(&self, doc: &UniversalDocument) -> Result<String, String> {
        self.check_token()?;

        // Stub — real impl: PUT https://graph.microsoft.com/v1.0/me/drive/items/{id}/content
        Ok(Self::markdown_to_ooxml(doc))
    }
}

// ─── Apple Notes Connector ───────────────────────────────────────────────────

/// Stub implementation of `SaaSConnector` for Apple Notes (via CloudKit).
///
/// Apple Notes does not have a public REST API; the real implementation would
/// use the CloudKit Web Services API with an Apple Developer token, or the
/// local macOS Notes.app AppleScript / XPC bridge.
pub struct AppleNoteConnector {
    /// Apple CloudKit API token or app-specific password.
    pub api_token: String,
}

impl AppleNoteConnector {
    /// Create a new connector with the given Apple CloudKit API token.
    pub fn new(api_token: impl Into<String>) -> Self {
        AppleNoteConnector { api_token: api_token.into() }
    }

    fn check_token(&self) -> Result<(), String> {
        if self.api_token.is_empty() {
            Err(String::from("Apple Notes: api_token is empty"))
        } else {
            Ok(())
        }
    }

    /// Simulate stripping Apple Notes XHTML into Markdown.
    ///
    /// Real implementation: parse the XHTML body stored in the `Note` CloudKit
    /// record, strip `<div>` / `<span>` tags, convert `<h1>`–`<h6>`, `<ul>`,
    /// `<ol>`, `<b>`, `<i>` into their Markdown equivalents.
    fn strip_xhtml_to_markdown(raw_title: &str, raw_body: &str) -> String {
        format!("# {raw_title}\n\n{raw_body}")
    }

    /// Simulate converting Markdown back into Apple Notes XHTML.
    ///
    /// Real implementation: produce a valid `text/html` body for a CloudKit
    /// `Note` record, wrapping Markdown headings in `<h1>`–`<h6>` tags and
    /// paragraphs in `<div>` tags.
    fn markdown_to_xhtml(doc: &UniversalDocument) -> String {
        format!(
            "<html><head><title>{title}</title></head><body><div>{body}</div></body></html>",
            title = doc.title,
            body = doc.body_markdown,
        )
    }
}

impl SaaSConnector for AppleNoteConnector {
    fn provider_name(&self) -> &str {
        "Apple Notes"
    }

    fn provider_format(&self) -> ProviderFormat {
        ProviderFormat::AppleNote
    }

    fn pull_and_strip(&self, document_id: &str) -> Result<UniversalDocument, String> {
        self.check_token()?;

        // Stub — real impl: CloudKit Web Services
        //   POST https://api.apple-cloudkit.com/database/1/{container}/development/private/records/lookup
        let title = format!("Apple Note {document_id}");
        let raw_body = "This note was retrieved from Apple Notes and stripped of XHTML formatting.";
        let body_md = Self::strip_xhtml_to_markdown(&title, raw_body);

        Ok(UniversalDocument::new(document_id, &title, body_md)
            .with_metadata("source", "Apple Notes")
            .with_metadata("connector_version", "stub-0.1"))
    }

    fn list_document_ids(&self) -> Result<Vec<String>, String> {
        self.check_token()?;

        // Stub — real impl: CloudKit query for all Note records
        Ok(vec![
            String::from("stub-note-id-001"),
            String::from("stub-note-id-002"),
        ])
    }

    fn repatriate(&self, doc: &UniversalDocument) -> Result<String, String> {
        self.check_token()?;

        // Stub — real impl: CloudKit record modify (save Note)
        Ok(Self::markdown_to_xhtml(doc))
    }
}

// ─── Plain Text Connector ────────────────────────────────────────────────────

/// Implementation of `SaaSConnector` for plain-text / Markdown files.
///
/// This connector reads documents from the local filesystem. It requires no
/// authentication token. It is the identity connector — `pull_and_strip` and
/// `repatriate` are both no-ops with respect to format conversion, making it
/// useful for testing and for local-first workflows.
pub struct PlainTextConnector {
    /// Base directory from which documents are read / written.
    pub base_dir: String,
}

impl PlainTextConnector {
    /// Create a new connector rooted at `base_dir`.
    pub fn new(base_dir: impl Into<String>) -> Self {
        PlainTextConnector { base_dir: base_dir.into() }
    }
}

impl SaaSConnector for PlainTextConnector {
    fn provider_name(&self) -> &str {
        "Plain Text"
    }

    fn provider_format(&self) -> ProviderFormat {
        ProviderFormat::PlainText
    }

    fn pull_and_strip(&self, document_id: &str) -> Result<UniversalDocument, String> {
        // Stub — real impl: read `{base_dir}/{document_id}.md` from disk.
        // No format stripping needed; Markdown is already the universal format.
        let body_md = format!("# {document_id}\n\nPlain text document stub.");

        Ok(UniversalDocument::new(document_id, document_id, body_md)
            .with_metadata("source", "Plain Text")
            .with_metadata("base_dir", &self.base_dir)
            .with_metadata("connector_version", "stub-0.1"))
    }

    fn list_document_ids(&self) -> Result<Vec<String>, String> {
        // Stub — real impl: list `*.md` files in `base_dir`.
        Ok(vec![
            String::from("stub-txt-id-001"),
            String::from("stub-txt-id-002"),
        ])
    }

    fn repatriate(&self, doc: &UniversalDocument) -> Result<String, String> {
        // Plain text is the universal format — repatriation is a no-op.
        // Real impl: write `{base_dir}/{doc.id}.md` to disk.
        Ok(doc.body_markdown.clone())
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── ProviderFormat ─────────────────────────────────────────────────────

    #[test]
    fn test_provider_format_mime_types() {
        assert_eq!(ProviderFormat::GoogleDoc.mime_type(), "application/vnd.google-apps.document");
        assert!(ProviderFormat::MicrosoftWord.mime_type().contains("wordprocessingml"));
        assert_eq!(ProviderFormat::AppleNote.mime_type(), "text/html");
        assert_eq!(ProviderFormat::PlainText.mime_type(), "text/plain");
    }

    #[test]
    fn test_provider_format_equality() {
        assert_eq!(ProviderFormat::GoogleDoc, ProviderFormat::GoogleDoc);
        assert_ne!(ProviderFormat::GoogleDoc, ProviderFormat::MicrosoftWord);
    }

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
    fn test_google_provider_name() {
        let c = GoogleWorkspaceConnector::new("token");
        assert_eq!(c.provider_name(), "Google Workspace");
        assert_eq!(c.provider_format(), ProviderFormat::GoogleDoc);
    }

    #[test]
    fn test_google_pull_and_strip_returns_document() {
        let c = GoogleWorkspaceConnector::new("fake-token");
        let doc = c.pull_and_strip("doc-abc").unwrap();
        assert_eq!(doc.id, "doc-abc");
        assert!(doc.body_markdown.contains("doc-abc"));
        assert_eq!(doc.metadata.get("source").map(String::as_str), Some("Google Workspace"));
    }

    #[test]
    fn test_google_pull_and_strip_empty_token_err() {
        let err = GoogleWorkspaceConnector::new("").pull_and_strip("doc-xyz").unwrap_err();
        assert!(err.contains("access_token is empty"));
    }

    #[test]
    fn test_google_list_document_ids() {
        let ids = GoogleWorkspaceConnector::new("token").list_document_ids().unwrap();
        assert!(!ids.is_empty());
    }

    #[test]
    fn test_google_repatriate_returns_json() {
        let c = GoogleWorkspaceConnector::new("token");
        let doc = UniversalDocument::new("d1", "Title", "Body");
        let result = c.repatriate(&doc).unwrap();
        assert!(result.contains("Title"));
        assert!(result.contains("Body"));
    }

    #[test]
    fn test_google_repatriate_empty_token_err() {
        let doc = UniversalDocument::new("d1", "Title", "Body");
        let err = GoogleWorkspaceConnector::new("").repatriate(&doc).unwrap_err();
        assert!(err.contains("access_token is empty"));
    }

    #[test]
    fn test_google_strip_to_markdown_format() {
        let md = GoogleWorkspaceConnector::strip_gdoc_to_markdown("My Doc", "Content.");
        assert!(md.starts_with("# My Doc"));
        assert!(md.contains("Content."));
    }

    // ── MicrosoftWordConnector ─────────────────────────────────────────────

    #[test]
    fn test_word_provider_name() {
        let c = MicrosoftWordConnector::new("token");
        assert_eq!(c.provider_name(), "Microsoft Word");
        assert_eq!(c.provider_format(), ProviderFormat::MicrosoftWord);
    }

    #[test]
    fn test_word_pull_and_strip_returns_document() {
        let doc = MicrosoftWordConnector::new("token").pull_and_strip("word-001").unwrap();
        assert_eq!(doc.id, "word-001");
        assert_eq!(doc.metadata.get("source").map(String::as_str), Some("Microsoft Word"));
    }

    #[test]
    fn test_word_pull_empty_token_err() {
        let err = MicrosoftWordConnector::new("").pull_and_strip("x").unwrap_err();
        assert!(err.contains("access_token is empty"));
    }

    #[test]
    fn test_word_list_document_ids() {
        let ids = MicrosoftWordConnector::new("token").list_document_ids().unwrap();
        assert!(!ids.is_empty());
    }

    #[test]
    fn test_word_repatriate_returns_ooxml() {
        let c = MicrosoftWordConnector::new("token");
        let doc = UniversalDocument::new("d1", "Title", "Body");
        let xml = c.repatriate(&doc).unwrap();
        assert!(xml.contains("<w:t>"));
        assert!(xml.contains("Title"));
    }

    #[test]
    fn test_word_repatriate_empty_token_err() {
        let doc = UniversalDocument::new("d1", "Title", "Body");
        let err = MicrosoftWordConnector::new("").repatriate(&doc).unwrap_err();
        assert!(err.contains("access_token is empty"));
    }

    // ── AppleNoteConnector ─────────────────────────────────────────────────

    #[test]
    fn test_apple_provider_name() {
        let c = AppleNoteConnector::new("token");
        assert_eq!(c.provider_name(), "Apple Notes");
        assert_eq!(c.provider_format(), ProviderFormat::AppleNote);
    }

    #[test]
    fn test_apple_pull_and_strip_returns_document() {
        let doc = AppleNoteConnector::new("token").pull_and_strip("note-001").unwrap();
        assert_eq!(doc.id, "note-001");
        assert_eq!(doc.metadata.get("source").map(String::as_str), Some("Apple Notes"));
    }

    #[test]
    fn test_apple_pull_empty_token_err() {
        let err = AppleNoteConnector::new("").pull_and_strip("x").unwrap_err();
        assert!(err.contains("api_token is empty"));
    }

    #[test]
    fn test_apple_list_document_ids() {
        let ids = AppleNoteConnector::new("token").list_document_ids().unwrap();
        assert!(!ids.is_empty());
    }

    #[test]
    fn test_apple_repatriate_returns_html() {
        let c = AppleNoteConnector::new("token");
        let doc = UniversalDocument::new("n1", "My Note", "Note content.");
        let html = c.repatriate(&doc).unwrap();
        assert!(html.contains("<html>"));
        assert!(html.contains("My Note"));
    }

    #[test]
    fn test_apple_repatriate_empty_token_err() {
        let doc = UniversalDocument::new("n1", "My Note", "body");
        let err = AppleNoteConnector::new("").repatriate(&doc).unwrap_err();
        assert!(err.contains("api_token is empty"));
    }

    // ── PlainTextConnector ─────────────────────────────────────────────────

    #[test]
    fn test_plain_provider_name() {
        let c = PlainTextConnector::new("/tmp/notes");
        assert_eq!(c.provider_name(), "Plain Text");
        assert_eq!(c.provider_format(), ProviderFormat::PlainText);
    }

    #[test]
    fn test_plain_pull_and_strip_returns_document() {
        let doc = PlainTextConnector::new("/tmp").pull_and_strip("readme").unwrap();
        assert_eq!(doc.id, "readme");
        assert_eq!(doc.metadata.get("source").map(String::as_str), Some("Plain Text"));
        assert_eq!(doc.metadata.get("base_dir").map(String::as_str), Some("/tmp"));
    }

    #[test]
    fn test_plain_list_document_ids() {
        let ids = PlainTextConnector::new("/tmp").list_document_ids().unwrap();
        assert!(!ids.is_empty());
    }

    #[test]
    fn test_plain_repatriate_is_identity() {
        let c = PlainTextConnector::new("/tmp");
        let doc = UniversalDocument::new("f1", "Title", "# Title\n\nBody.");
        let out = c.repatriate(&doc).unwrap();
        // PlainText repatriate is a pass-through — Markdown is the universal format.
        assert_eq!(out, doc.body_markdown);
    }

    // ── Cross-connector round-trip ─────────────────────────────────────────

    #[test]
    fn test_roundtrip_google_repatriate_preserves_title() {
        let c = GoogleWorkspaceConnector::new("token");
        let original = c.pull_and_strip("my-doc").unwrap();
        let repatriated = c.repatriate(&original).unwrap();
        assert!(repatriated.contains(&original.title));
    }

    #[test]
    fn test_roundtrip_word_repatriate_preserves_title() {
        let c = MicrosoftWordConnector::new("token");
        let original = c.pull_and_strip("my-word-doc").unwrap();
        let repatriated = c.repatriate(&original).unwrap();
        assert!(repatriated.contains(&original.title));
    }
}
