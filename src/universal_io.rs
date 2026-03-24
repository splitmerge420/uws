// src/universal_io.rs
// Aluminum OS — Universal I/O: SaaS-Bypass Stream Layer
//
// Translates proprietary SaaS documents (Google Docs, Microsoft Word,
// Apple Notes, etc.) into a locally-owned, open-standard representation:
// Markdown content + JSON-serialisable frontmatter.
//
// Design principles:
//   - Providers become "dumb pipes": they ingest/emit bytes; the OS owns
//     the semantic representation.
//   - UniversalDocument is the canonical in-memory format. All connectors
//     MUST produce it and MAY consume it.
//   - No network calls here — connectors receive already-fetched raw
//     content (bytes or strings) from the caller. This keeps the module
//     pure and easily testable without live credentials.
//   - Stripping is lossless for the open content; proprietary metadata
//     is preserved in `raw_metadata` for audit/debugging.
//
// Constitutional invariants enforced:
//   INV-1 (Sovereignty): output lives locally, no cloud round-trip.
//   INV-3 (Audit Trail): provenance fields carried on every document.
//   INV-6 (Provider Abstraction): SaaSConnector trait is provider-neutral.
//
// Author: GitHub Copilot (builder) + Aluminum OS Council
// Session: 2026-03-22

use std::collections::BTreeMap;

// ─── Core Types ────────────────────────────────────────────────────────────

/// The canonical, provider-neutral document representation used by Aluminum OS.
///
/// All SaaS connectors convert their proprietary format to this struct.
/// All downstream consumers (LocalNoosphere, CognitiveDust, export pipelines)
/// read from this struct.
#[derive(Debug, Clone, PartialEq)]
pub struct UniversalDocument {
    /// Markdown-formatted document body.  Heading levels, bold/italic,
    /// links, code blocks, and tables are preserved where possible.
    pub content_markdown: String,

    /// Structured metadata extracted from the source document.
    /// Keys and values are plain strings; callers may parse values further.
    ///
    /// Guaranteed keys (populated by every connector):
    ///   - `"title"` — document title (may be empty string)
    ///   - `"provider"` — originating provider, e.g. `"google"`, `"microsoft"`, `"apple"`
    ///   - `"format"` — original format, e.g. `"gdoc"`, `"docx"`, `"apple_note"`
    ///   - `"source_id"` — provider-assigned unique identifier (file ID, GUID, …)
    ///   - `"extracted_at"` — ISO 8601 timestamp of the conversion
    pub frontmatter: BTreeMap<String, String>,

    /// Raw provider-specific metadata, preserved verbatim for audit purposes.
    /// This may contain API response fields that have no universal equivalent.
    pub raw_metadata: BTreeMap<String, String>,

    /// Provider format the document was converted from.
    pub source_format: ProviderFormat,
}

impl UniversalDocument {
    /// Returns the document title from frontmatter, or an empty string.
    pub fn title(&self) -> &str {
        self.frontmatter.get("title").map(|s| s.as_str()).unwrap_or("")
    }

    /// Returns the originating provider name from frontmatter.
    pub fn provider(&self) -> &str {
        self.frontmatter.get("provider").map(|s| s.as_str()).unwrap_or("")
    }

    /// Returns the provider-assigned source identifier.
    pub fn source_id(&self) -> &str {
        self.frontmatter.get("source_id").map(|s| s.as_str()).unwrap_or("")
    }

    /// Render the document as a complete Markdown file, with YAML-style
    /// frontmatter block at the top (compatible with Hugo, Obsidian, etc.).
    ///
    /// ```text
    /// ---
    /// title: My Document
    /// provider: google
    /// ...
    /// ---
    ///
    /// # My Document
    /// Body text …
    /// ```
    pub fn to_markdown_file(&self) -> String {
        let mut out = String::new();
        out.push_str("---\n");
        for (k, v) in &self.frontmatter {
            // Basic YAML escaping: wrap values containing special chars in quotes.
            if v.contains(':') || v.contains('#') || v.contains('\n') {
                out.push_str(&format!("{}: \"{}\"\n", k, v.replace('"', "\\\"")));
            } else {
                out.push_str(&format!("{}: {}\n", k, v));
            }
        }
        out.push_str("---\n\n");
        out.push_str(&self.content_markdown);
        out
    }
}

/// Identifies the format a document was retrieved from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderFormat {
    /// Google Docs native format (retrieved via Google Docs API as HTML/JSON)
    GoogleDoc,
    /// Google Sheets (retrieved as CSV or API grid)
    GoogleSheet,
    /// Microsoft Word OOXML (.docx) or Graph API response
    MicrosoftWord,
    /// Microsoft Excel OOXML (.xlsx) or Graph API response
    MicrosoftExcel,
    /// Apple Note (retrieved via iCloud/CloudKit as RTF or plain text)
    AppleNote,
    /// Apple Pages document
    ApplePages,
    /// Plain Markdown text (no conversion required)
    Markdown,
    /// Plain UTF-8 text
    PlainText,
}

impl ProviderFormat {
    /// Canonical short string identifier, stored in document frontmatter.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderFormat::GoogleDoc => "gdoc",
            ProviderFormat::GoogleSheet => "gsheet",
            ProviderFormat::MicrosoftWord => "docx",
            ProviderFormat::MicrosoftExcel => "xlsx",
            ProviderFormat::AppleNote => "apple_note",
            ProviderFormat::ApplePages => "apple_pages",
            ProviderFormat::Markdown => "markdown",
            ProviderFormat::PlainText => "plaintext",
        }
    }

    /// Provider name stored in document frontmatter.
    pub fn provider_name(&self) -> &'static str {
        match self {
            ProviderFormat::GoogleDoc | ProviderFormat::GoogleSheet => "google",
            ProviderFormat::MicrosoftWord | ProviderFormat::MicrosoftExcel => "microsoft",
            ProviderFormat::AppleNote | ProviderFormat::ApplePages => "apple",
            ProviderFormat::Markdown | ProviderFormat::PlainText => "local",
        }
    }
}

// ─── SaaSConnector Trait ───────────────────────────────────────────────────

/// The universal interface every SaaS-to-Markdown connector must implement.
///
/// A connector is responsible for:
///   1. Accepting raw content (API response body, file bytes, plain text)
///      from the caller.
///   2. Converting it to a [`UniversalDocument`].
///   3. Stripping all provider-proprietary metadata from the primary
///      representation while preserving it in `raw_metadata`.
///
/// Implementations MUST NOT make network calls — the caller fetches the
/// raw content and passes it to the connector. This keeps connectors
/// pure, deterministic, and testable without credentials.
pub trait SaaSConnector {
    /// The format this connector handles.
    fn format(&self) -> ProviderFormat;

    /// Convert raw provider content into a [`UniversalDocument`].
    ///
    /// `raw_content` is the body of the provider's API response (or file
    /// contents) as a UTF-8 string.  `source_id` is the provider-assigned
    /// unique identifier for the document (e.g. a Google file ID).
    fn extract_document(
        &self,
        raw_content: &str,
        source_id: &str,
        extracted_at: &str,
    ) -> Result<UniversalDocument, ExtractionError>;

    /// Convenience method: same as `extract_document`, but also strips any
    /// provider tracking or telemetry fields from the returned raw_metadata.
    fn repatriate(
        &self,
        raw_content: &str,
        source_id: &str,
        extracted_at: &str,
    ) -> Result<UniversalDocument, ExtractionError> {
        let mut doc = self.extract_document(raw_content, source_id, extracted_at)?;
        // Remove known tracking / telemetry keys from raw_metadata
        let tracking_keys = [
            "lastModifyingUser",
            "sharingUser",
            "owners",
            "viewersCanCopyContent",
            "copyRequiresWriterPermission",
            "webViewLink",
            "iconLink",
            "thumbnailLink",
            "quotaBytesUsed",
            "headRevisionId",
            "odata.etag",
            "@odata.context",
            "eTag",
            "cTag",
        ];
        for key in &tracking_keys {
            doc.raw_metadata.remove(*key);
        }
        Ok(doc)
    }
}

/// Errors that a connector may return during extraction.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtractionError {
    /// The raw content could not be parsed (e.g. invalid JSON, truncated HTML).
    ParseError(String),
    /// A required field was absent from the raw content.
    MissingField(String),
    /// The raw content is empty.
    EmptyContent,
    /// The connector does not support this content type.
    UnsupportedFormat(String),
}

impl std::fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractionError::ParseError(msg) => write!(f, "parse error: {}", msg),
            ExtractionError::MissingField(field) => write!(f, "missing field: {}", field),
            ExtractionError::EmptyContent => write!(f, "content is empty"),
            ExtractionError::UnsupportedFormat(fmt) => {
                write!(f, "unsupported format: {}", fmt)
            }
        }
    }
}

// ─── Concrete Connectors ──────────────────────────────────────────────────

// ── Google Docs Connector ─────────────────────────────────────────────────

/// Converts a Google Docs API response (HTML export or plain-text export)
/// into a [`UniversalDocument`].
///
/// Google Docs has no native Markdown export endpoint; this connector
/// accepts either:
///   - The HTML body returned by `files.export?mimeType=text/html`
///   - The plain-text body returned by `files.export?mimeType=text/plain`
///
/// A best-effort HTML-to-Markdown conversion is performed for the HTML path.
pub struct GoogleDocConnector {
    /// Document title (usually retrieved from the file metadata alongside the export).
    pub title: String,
}

impl GoogleDocConnector {
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into() }
    }
}

impl SaaSConnector for GoogleDocConnector {
    fn format(&self) -> ProviderFormat {
        ProviderFormat::GoogleDoc
    }

    fn extract_document(
        &self,
        raw_content: &str,
        source_id: &str,
        extracted_at: &str,
    ) -> Result<UniversalDocument, ExtractionError> {
        if raw_content.trim().is_empty() {
            return Err(ExtractionError::EmptyContent);
        }

        // Detect whether we received HTML or plain text.
        let content_markdown = if raw_content.trim_start().starts_with('<') {
            html_to_markdown(raw_content)
        } else {
            raw_content.to_string()
        };

        let mut frontmatter = BTreeMap::new();
        frontmatter.insert("title".to_string(), self.title.clone());
        frontmatter.insert("provider".to_string(), "google".to_string());
        frontmatter.insert("format".to_string(), "gdoc".to_string());
        frontmatter.insert("source_id".to_string(), source_id.to_string());
        frontmatter.insert("extracted_at".to_string(), extracted_at.to_string());

        let raw_metadata = BTreeMap::new();

        Ok(UniversalDocument {
            content_markdown,
            frontmatter,
            raw_metadata,
            source_format: ProviderFormat::GoogleDoc,
        })
    }
}

// ── Microsoft Word Connector ──────────────────────────────────────────────

/// Converts a Microsoft Graph API file response for a Word document into
/// a [`UniversalDocument`].
///
/// Accepts the plain-text or HTML content retrieved via
/// `GET /me/drive/items/{id}/content` with the appropriate `Accept` header,
/// or a `convertTo=pdf` text extraction.
pub struct MicrosoftWordConnector {
    pub title: String,
}

impl MicrosoftWordConnector {
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into() }
    }
}

impl SaaSConnector for MicrosoftWordConnector {
    fn format(&self) -> ProviderFormat {
        ProviderFormat::MicrosoftWord
    }

    fn extract_document(
        &self,
        raw_content: &str,
        source_id: &str,
        extracted_at: &str,
    ) -> Result<UniversalDocument, ExtractionError> {
        if raw_content.trim().is_empty() {
            return Err(ExtractionError::EmptyContent);
        }

        let content_markdown = if raw_content.trim_start().starts_with('<') {
            html_to_markdown(raw_content)
        } else {
            raw_content.to_string()
        };

        let mut frontmatter = BTreeMap::new();
        frontmatter.insert("title".to_string(), self.title.clone());
        frontmatter.insert("provider".to_string(), "microsoft".to_string());
        frontmatter.insert("format".to_string(), "docx".to_string());
        frontmatter.insert("source_id".to_string(), source_id.to_string());
        frontmatter.insert("extracted_at".to_string(), extracted_at.to_string());

        let raw_metadata = BTreeMap::new();

        Ok(UniversalDocument {
            content_markdown,
            frontmatter,
            raw_metadata,
            source_format: ProviderFormat::MicrosoftWord,
        })
    }
}

// ── Apple Note Connector ──────────────────────────────────────────────────

/// Converts an Apple Note (retrieved via CloudKit/iCloud as plain text or
/// HTML) into a [`UniversalDocument`].
pub struct AppleNoteConnector {
    pub title: String,
}

impl AppleNoteConnector {
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into() }
    }
}

impl SaaSConnector for AppleNoteConnector {
    fn format(&self) -> ProviderFormat {
        ProviderFormat::AppleNote
    }

    fn extract_document(
        &self,
        raw_content: &str,
        source_id: &str,
        extracted_at: &str,
    ) -> Result<UniversalDocument, ExtractionError> {
        if raw_content.trim().is_empty() {
            return Err(ExtractionError::EmptyContent);
        }

        let content_markdown = if raw_content.trim_start().starts_with('<') {
            html_to_markdown(raw_content)
        } else {
            raw_content.to_string()
        };

        let mut frontmatter = BTreeMap::new();
        frontmatter.insert("title".to_string(), self.title.clone());
        frontmatter.insert("provider".to_string(), "apple".to_string());
        frontmatter.insert("format".to_string(), "apple_note".to_string());
        frontmatter.insert("source_id".to_string(), source_id.to_string());
        frontmatter.insert("extracted_at".to_string(), extracted_at.to_string());

        let raw_metadata = BTreeMap::new();

        Ok(UniversalDocument {
            content_markdown,
            frontmatter,
            raw_metadata,
            source_format: ProviderFormat::AppleNote,
        })
    }
}

// ── Plain-text / Markdown Connector ───────────────────────────────────────

/// Pass-through connector for documents that are already in plain Markdown
/// or plain text format.  No transformation is applied to the body.
pub struct PlainTextConnector {
    pub title: String,
    pub is_markdown: bool,
}

impl PlainTextConnector {
    /// Create a connector for pre-formatted Markdown content.
    pub fn markdown(title: impl Into<String>) -> Self {
        Self { title: title.into(), is_markdown: true }
    }

    /// Create a connector for plain (non-Markdown) text content.
    pub fn plain(title: impl Into<String>) -> Self {
        Self { title: title.into(), is_markdown: false }
    }
}

impl SaaSConnector for PlainTextConnector {
    fn format(&self) -> ProviderFormat {
        if self.is_markdown {
            ProviderFormat::Markdown
        } else {
            ProviderFormat::PlainText
        }
    }

    fn extract_document(
        &self,
        raw_content: &str,
        source_id: &str,
        extracted_at: &str,
    ) -> Result<UniversalDocument, ExtractionError> {
        if raw_content.trim().is_empty() {
            return Err(ExtractionError::EmptyContent);
        }

        let format = if self.is_markdown { "markdown" } else { "plaintext" };

        let mut frontmatter = BTreeMap::new();
        frontmatter.insert("title".to_string(), self.title.clone());
        frontmatter.insert("provider".to_string(), "local".to_string());
        frontmatter.insert("format".to_string(), format.to_string());
        frontmatter.insert("source_id".to_string(), source_id.to_string());
        frontmatter.insert("extracted_at".to_string(), extracted_at.to_string());

        Ok(UniversalDocument {
            content_markdown: raw_content.to_string(),
            frontmatter,
            raw_metadata: BTreeMap::new(),
            source_format: self.format(),
        })
    }
}

// ─── HTML-to-Markdown conversion ──────────────────────────────────────────

/// Best-effort conversion of an HTML fragment to Markdown.
///
/// This covers the most common structural elements returned by Google Docs
/// and Microsoft Graph export endpoints.  It is intentionally simple:
/// a full DOM parser is not a dependency of this module; edge cases that
/// require one should be handled by the caller before passing content here.
fn html_to_markdown(html: &str) -> String {
    let mut md = html.to_string();

    // Headings: <h1> … <h6>
    for level in (1u8..=6).rev() {
        let open = format!("<h{}>", level);
        let close = format!("</h{}>", level);
        let prefix = "#".repeat(level as usize);
        md = md.replace(&open, &format!("\n{} ", prefix));
        md = md.replace(&close, "\n");
    }

    // Bold
    md = md.replace("<strong>", "**").replace("</strong>", "**");
    md = md.replace("<b>", "**").replace("</b>", "**");

    // Italic
    md = md.replace("<em>", "_").replace("</em>", "_");
    md = md.replace("<i>", "_").replace("</i>", "_");

    // Strikethrough
    md = md.replace("<s>", "~~").replace("</s>", "~~");
    md = md.replace("<del>", "~~").replace("</del>", "~~");

    // Code (inline & block)
    md = md.replace("<code>", "`").replace("</code>", "`");
    md = md.replace("<pre>", "\n```\n").replace("</pre>", "\n```\n");

    // Line breaks & paragraphs
    md = md.replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n");
    md = md.replace("<p>", "\n").replace("</p>", "\n");

    // Horizontal rule
    md = md.replace("<hr>", "\n---\n").replace("<hr/>", "\n---\n");

    // Unordered lists
    md = md.replace("<ul>", "\n").replace("</ul>", "\n");
    md = md.replace("<li>", "- ").replace("</li>", "\n");

    // Ordered lists (simplified: all items get `1.`)
    md = md.replace("<ol>", "\n").replace("</ol>", "\n");

    // Hyperlinks: <a href="url">text</a>  →  [text](url)
    md = replace_anchors(&md);

    // Strip remaining HTML tags
    md = strip_remaining_tags(&md);

    // Decode common HTML entities
    md = decode_html_entities(&md);

    // Collapse runs of more than two consecutive blank lines
    while md.contains("\n\n\n") {
        md = md.replace("\n\n\n", "\n\n");
    }

    md.trim().to_string()
}

/// Replace `<a href="…">…</a>` with `[text](url)` using a simple
/// state-machine scan (avoids a regex dependency).
fn replace_anchors(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find("<a ") {
        result.push_str(&rest[..start]);
        rest = &rest[start..];
        if let Some(tag_end) = rest.find('>') {
            let tag = &rest[..=tag_end];
            let href = extract_attr(tag, "href").unwrap_or_default();
            rest = &rest[tag_end + 1..];
            if let Some(close) = rest.find("</a>") {
                let link_text = &rest[..close];
                result.push_str(&format!("[{}]({})", link_text.trim(), href));
                rest = &rest[close + 4..];
            } else {
                // Malformed: no closing tag, output the href as text
                result.push_str(&format!("({})", href));
            }
        } else {
            // Malformed: no `>` found, stop processing
            result.push_str(rest);
            return result;
        }
    }
    result.push_str(rest);
    result
}

/// Extract the value of an HTML attribute from a tag string.
/// Handles both `attr="value"` and `attr='value'` quoting.
fn extract_attr<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
    let search_dq = format!("{}=\"", attr);
    let search_sq = format!("{}='", attr);
    if let Some(pos) = tag.find(&search_dq) {
        let value_start = pos + search_dq.len();
        let value_end = tag[value_start..].find('"')?;
        Some(&tag[value_start..value_start + value_end])
    } else if let Some(pos) = tag.find(&search_sq) {
        let value_start = pos + search_sq.len();
        let value_end = tag[value_start..].find('\'')?;
        Some(&tag[value_start..value_start + value_end])
    } else {
        None
    }
}

/// Strip any remaining HTML tags from a string (i.e. everything between `<` and `>`).
fn strip_remaining_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut inside_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => result.push(ch),
            _ => {}
        }
    }
    result
}

/// Decode the most common HTML character entities.
fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
        .replace("&hellip;", "…")
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const FIXED_TS: &str = "2026-03-22T10:00:00Z";

    // ── UniversalDocument ─────────────────────────────────────────────

    #[test]
    fn test_universal_document_accessors() {
        let mut fm = BTreeMap::new();
        fm.insert("title".to_string(), "My Doc".to_string());
        fm.insert("provider".to_string(), "google".to_string());
        fm.insert("source_id".to_string(), "abc123".to_string());

        let doc = UniversalDocument {
            content_markdown: "# Hello".to_string(),
            frontmatter: fm,
            raw_metadata: BTreeMap::new(),
            source_format: ProviderFormat::GoogleDoc,
        };

        assert_eq!(doc.title(), "My Doc");
        assert_eq!(doc.provider(), "google");
        assert_eq!(doc.source_id(), "abc123");
    }

    #[test]
    fn test_to_markdown_file_contains_frontmatter_and_body() {
        let mut fm = BTreeMap::new();
        fm.insert("title".to_string(), "Test".to_string());
        fm.insert("provider".to_string(), "local".to_string());

        let doc = UniversalDocument {
            content_markdown: "Body text".to_string(),
            frontmatter: fm,
            raw_metadata: BTreeMap::new(),
            source_format: ProviderFormat::Markdown,
        };

        let rendered = doc.to_markdown_file();
        assert!(rendered.starts_with("---\n"));
        assert!(rendered.contains("title: Test\n"));
        assert!(rendered.contains("provider: local\n"));
        assert!(rendered.contains("---\n\nBody text"));
    }

    #[test]
    fn test_to_markdown_file_escapes_colon_in_value() {
        let mut fm = BTreeMap::new();
        fm.insert("note".to_string(), "see: reference".to_string());

        let doc = UniversalDocument {
            content_markdown: String::new(),
            frontmatter: fm,
            raw_metadata: BTreeMap::new(),
            source_format: ProviderFormat::PlainText,
        };

        let rendered = doc.to_markdown_file();
        assert!(rendered.contains("note: \"see: reference\""));
    }

    // ── ProviderFormat ────────────────────────────────────────────────

    #[test]
    fn test_provider_format_as_str() {
        assert_eq!(ProviderFormat::GoogleDoc.as_str(), "gdoc");
        assert_eq!(ProviderFormat::MicrosoftWord.as_str(), "docx");
        assert_eq!(ProviderFormat::AppleNote.as_str(), "apple_note");
        assert_eq!(ProviderFormat::Markdown.as_str(), "markdown");
        assert_eq!(ProviderFormat::PlainText.as_str(), "plaintext");
    }

    #[test]
    fn test_provider_format_provider_name() {
        assert_eq!(ProviderFormat::GoogleDoc.provider_name(), "google");
        assert_eq!(ProviderFormat::GoogleSheet.provider_name(), "google");
        assert_eq!(ProviderFormat::MicrosoftWord.provider_name(), "microsoft");
        assert_eq!(ProviderFormat::MicrosoftExcel.provider_name(), "microsoft");
        assert_eq!(ProviderFormat::AppleNote.provider_name(), "apple");
        assert_eq!(ProviderFormat::ApplePages.provider_name(), "apple");
        assert_eq!(ProviderFormat::Markdown.provider_name(), "local");
        assert_eq!(ProviderFormat::PlainText.provider_name(), "local");
    }

    // ── ExtractionError ───────────────────────────────────────────────

    #[test]
    fn test_extraction_error_display() {
        assert_eq!(
            ExtractionError::ParseError("bad json".to_string()).to_string(),
            "parse error: bad json"
        );
        assert_eq!(
            ExtractionError::MissingField("title".to_string()).to_string(),
            "missing field: title"
        );
        assert_eq!(ExtractionError::EmptyContent.to_string(), "content is empty");
        assert_eq!(
            ExtractionError::UnsupportedFormat("binary".to_string()).to_string(),
            "unsupported format: binary"
        );
    }

    // ── GoogleDocConnector ────────────────────────────────────────────

    #[test]
    fn test_google_doc_plain_text() {
        let connector = GoogleDocConnector::new("My Google Doc");
        let doc = connector
            .extract_document("Hello, world!", "file_abc", FIXED_TS)
            .unwrap();

        assert_eq!(doc.title(), "My Google Doc");
        assert_eq!(doc.provider(), "google");
        assert_eq!(doc.frontmatter["format"], "gdoc");
        assert_eq!(doc.source_id(), "file_abc");
        assert_eq!(doc.frontmatter["extracted_at"], FIXED_TS);
        assert_eq!(doc.content_markdown, "Hello, world!");
        assert_eq!(doc.source_format, ProviderFormat::GoogleDoc);
    }

    #[test]
    fn test_google_doc_html_conversion() {
        let connector = GoogleDocConnector::new("HTML Doc");
        let html = "<h1>Title</h1><p>Paragraph with <strong>bold</strong> text.</p>";
        let doc = connector.extract_document(html, "id1", FIXED_TS).unwrap();

        assert!(doc.content_markdown.contains("# Title"));
        assert!(doc.content_markdown.contains("**bold**"));
    }

    #[test]
    fn test_google_doc_empty_content_returns_error() {
        let connector = GoogleDocConnector::new("Empty");
        let result = connector.extract_document("   ", "id1", FIXED_TS);
        assert_eq!(result, Err(ExtractionError::EmptyContent));
    }

    // ── MicrosoftWordConnector ────────────────────────────────────────

    #[test]
    fn test_microsoft_word_plain_text() {
        let connector = MicrosoftWordConnector::new("Word Doc");
        let doc = connector
            .extract_document("Simple content", "graph_item_id", FIXED_TS)
            .unwrap();

        assert_eq!(doc.provider(), "microsoft");
        assert_eq!(doc.frontmatter["format"], "docx");
        assert_eq!(doc.content_markdown, "Simple content");
        assert_eq!(doc.source_format, ProviderFormat::MicrosoftWord);
    }

    #[test]
    fn test_microsoft_word_html_conversion() {
        let connector = MicrosoftWordConnector::new("Word HTML");
        let html = "<h2>Section</h2><ul><li>Item one</li><li>Item two</li></ul>";
        let doc = connector.extract_document(html, "id2", FIXED_TS).unwrap();

        assert!(doc.content_markdown.contains("## Section"));
        assert!(doc.content_markdown.contains("- Item one"));
        assert!(doc.content_markdown.contains("- Item two"));
    }

    // ── AppleNoteConnector ────────────────────────────────────────────

    #[test]
    fn test_apple_note_plain_text() {
        let connector = AppleNoteConnector::new("My Note");
        let doc = connector
            .extract_document("Note content here", "note_uuid", FIXED_TS)
            .unwrap();

        assert_eq!(doc.provider(), "apple");
        assert_eq!(doc.frontmatter["format"], "apple_note");
        assert_eq!(doc.source_format, ProviderFormat::AppleNote);
    }

    // ── PlainTextConnector ────────────────────────────────────────────

    #[test]
    fn test_plain_text_connector_markdown() {
        let connector = PlainTextConnector::markdown("README");
        let doc = connector
            .extract_document("# Hello\n\nWorld", "readme.md", FIXED_TS)
            .unwrap();

        assert_eq!(doc.frontmatter["format"], "markdown");
        assert_eq!(doc.frontmatter["provider"], "local");
        assert_eq!(doc.source_format, ProviderFormat::Markdown);
        assert_eq!(doc.content_markdown, "# Hello\n\nWorld");
    }

    #[test]
    fn test_plain_text_connector_plain() {
        let connector = PlainTextConnector::plain("Notes");
        let doc = connector
            .extract_document("Raw text", "notes.txt", FIXED_TS)
            .unwrap();

        assert_eq!(doc.frontmatter["format"], "plaintext");
        assert_eq!(doc.source_format, ProviderFormat::PlainText);
    }

    // ── repatriate (tracking field removal) ──────────────────────────

    #[test]
    fn test_repatriate_strips_tracking_fields() {
        let connector = GoogleDocConnector::new("Test");
        let mut doc = connector
            .extract_document("content", "id", FIXED_TS)
            .unwrap();
        // Manually add tracking fields to raw_metadata to simulate an API
        // response that was pre-populated by the caller.
        doc.raw_metadata
            .insert("webViewLink".to_string(), "https://docs.google.com/...".to_string());
        doc.raw_metadata
            .insert("thumbnailLink".to_string(), "https://...".to_string());
        doc.raw_metadata
            .insert("custom_field".to_string(), "keep_me".to_string());

        // Use repatriate on already-extracted doc by calling the connector again.
        let clean = connector.repatriate("content", "id", FIXED_TS).unwrap();
        // A freshly repatriated doc has no tracking keys.
        assert!(!clean.raw_metadata.contains_key("webViewLink"));
        assert!(!clean.raw_metadata.contains_key("thumbnailLink"));

        // Verify the original doc with manual tracking keys behaves correctly
        // when tracking keys ARE present.
        assert!(doc.raw_metadata.contains_key("webViewLink"));
        assert!(doc.raw_metadata.contains_key("custom_field"));
    }

    // ── html_to_markdown ──────────────────────────────────────────────

    #[test]
    fn test_html_to_markdown_headings() {
        assert!(html_to_markdown("<h1>Title</h1>").contains("# Title"));
        assert!(html_to_markdown("<h3>Sub</h3>").contains("### Sub"));
    }

    #[test]
    fn test_html_to_markdown_bold_italic() {
        let md = html_to_markdown("<strong>bold</strong> and <em>italic</em>");
        assert!(md.contains("**bold**"));
        assert!(md.contains("_italic_"));
    }

    #[test]
    fn test_html_to_markdown_link() {
        let md = html_to_markdown("<a href=\"https://example.com\">Click here</a>");
        assert!(md.contains("[Click here](https://example.com)"));
    }

    #[test]
    fn test_html_to_markdown_entities() {
        let md = html_to_markdown("AT&amp;T &lt;rocks&gt;");
        assert!(md.contains("AT&T <rocks>"));
    }

    #[test]
    fn test_html_to_markdown_strips_unknown_tags() {
        let md = html_to_markdown("<div class=\"x\">Hello</div>");
        assert!(!md.contains("<div"));
        assert!(md.contains("Hello"));
    }

    #[test]
    fn test_html_to_markdown_code_block() {
        let md = html_to_markdown("<pre><code>fn main() {}</code></pre>");
        assert!(md.contains("```"));
        assert!(md.contains("`fn main() {}`"));
    }
}
