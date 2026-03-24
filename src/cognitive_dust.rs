// src/cognitive_dust.rs
// Aluminum OS — CognitiveDust: SaaS Data Repatriation Sweeper
//
// A structured sweep engine that ingests documents from connected SaaS
// providers (Google Drive, Microsoft OneDrive, Apple Notes, etc.),
// converts them to the sovereign `UniversalDocument` format via the
// `universal_io` connectors, and writes them into the local `Noosphere`.
//
// Philosophy:
//   SaaS platforms become write-once inboxes.  Your OS is the permanent
//   record.  CognitiveDust is the automated janitor that enforces this.
//
// Architecture:
//   - `SweepConfig` describes what to sweep (provider, filters, delete policy).
//   - `SweepManifest` is an append-only record of what has already been swept
//     (enables incremental sweeps — skip docs ingested since last run).
//   - `SweepResult` summarises one sweep run.
//   - `DustSweeper` is the main engine; it accepts pre-fetched raw content
//     (same design philosophy as `universal_io` — no network calls here)
//     and processes it into `Noosphere` entries.
//
// Constitutional invariants:
//   INV-1 (Sovereignty)  — local copy exists before any remote deletion.
//   INV-2 (Consent)      — `delete_remote` is opt-in, off by default.
//   INV-3 (Audit Trail)  — `SweepManifest` records every swept document.
//
// Author: GitHub Copilot (builder) + Aluminum OS Council
// Session: 2026-03-22

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use crate::local_noosphere::{Noosphere, NoosphereEntry, ResourceType};
use crate::universal_io::{
    AppleNoteConnector, GoogleDocConnector, MicrosoftWordConnector, PlainTextConnector,
    ProviderFormat, SaaSConnector,
};

// ─── Config ───────────────────────────────────────────────────────────────

/// Which provider(s) to sweep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SweepTarget {
    Google,
    Microsoft,
    Apple,
    Local,
    All,
}

impl SweepTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            SweepTarget::Google => "google",
            SweepTarget::Microsoft => "microsoft",
            SweepTarget::Apple => "apple",
            SweepTarget::Local => "local",
            SweepTarget::All => "all",
        }
    }
}

/// Configuration for a single sweep run.
#[derive(Debug, Clone)]
pub struct SweepConfig {
    /// Which provider(s) to sweep.
    pub target: SweepTarget,
    /// If `true`, delete the remote copy after successful local ingestion.
    /// **Off by default — consent required (INV-2).**
    pub delete_remote: bool,
    /// Only sweep documents modified after this ISO 8601 timestamp.
    /// `None` = sweep all.
    pub since: Option<String>,
    /// Maximum number of documents to process per provider per run.
    /// `None` = no limit.
    pub limit: Option<usize>,
    /// Tags to apply to every document ingested in this sweep.
    pub tags: Vec<String>,
    /// Dry run: process documents but do not write to the Noosphere.
    pub dry_run: bool,
}

impl SweepConfig {
    pub fn new(target: SweepTarget) -> Self {
        SweepConfig {
            target,
            delete_remote: false,
            since: None,
            limit: None,
            tags: vec![],
            dry_run: false,
        }
    }

    pub fn with_delete_remote(mut self, v: bool) -> Self {
        self.delete_remote = v;
        self
    }

    pub fn with_since(mut self, ts: impl Into<String>) -> Self {
        self.since = Some(ts.into());
        self
    }

    pub fn with_limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }
}

// ─── Manifest ─────────────────────────────────────────────────────────────

/// A record of a single document that was successfully swept.
#[derive(Debug, Clone)]
pub struct ManifestEntry {
    /// The `NoosphereEntry` ID assigned to this document.
    pub noosphere_id: String,
    /// Provider name.
    pub provider: String,
    /// Provider-assigned source ID.
    pub source_id: String,
    /// ISO 8601 timestamp of when this document was swept.
    pub swept_at: String,
    /// Whether the remote copy was deleted.
    pub remote_deleted: bool,
}

/// Append-only record of all documents swept across all runs.
/// Used for incremental sweeps: if a source_id is already in the manifest,
/// and the document has not been modified since, it is skipped.
#[derive(Debug, Default)]
pub struct SweepManifest {
    entries: BTreeMap<String, ManifestEntry>, // keyed by noosphere_id
    source_ids_seen: BTreeSet<String>,        // "<provider>:<source_id>"
}

impl SweepManifest {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that a document was swept.
    pub fn record(
        &mut self,
        noosphere_id: impl Into<String>,
        provider: impl Into<String>,
        source_id: impl Into<String>,
        swept_at: impl Into<String>,
        remote_deleted: bool,
    ) {
        let nid = noosphere_id.into();
        let prov = provider.into();
        let sid = source_id.into();
        let key = format!("{}:{}", prov, sid);
        self.source_ids_seen.insert(key);
        self.entries.insert(
            nid.clone(),
            ManifestEntry {
                noosphere_id: nid,
                provider: prov,
                source_id: sid,
                swept_at: swept_at.into(),
                remote_deleted,
            },
        );
    }

    /// Returns `true` if this source document has already been swept.
    pub fn already_swept(&self, provider: &str, source_id: &str) -> bool {
        let key = format!("{}:{}", provider, source_id);
        self.source_ids_seen.contains(&key)
    }

    /// Number of documents recorded in the manifest.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Entries for a specific provider.
    pub fn by_provider(&self, provider: &str) -> Vec<&ManifestEntry> {
        self.entries
            .values()
            .filter(|e| e.provider == provider)
            .collect()
    }
}

// ─── Sweep result ─────────────────────────────────────────────────────────

/// Summary of a single sweep run.
#[derive(Debug, Default, Clone)]
pub struct SweepResult {
    pub provider: String,
    pub docs_found: usize,
    pub docs_ingested: usize,
    pub docs_skipped: usize,
    pub docs_failed: usize,
    pub remote_deleted: usize,
    pub errors: Vec<String>,
    pub dry_run: bool,
}

impl SweepResult {
    pub fn new(provider: impl Into<String>, dry_run: bool) -> Self {
        SweepResult {
            provider: provider.into(),
            dry_run,
            ..Default::default()
        }
    }

    pub fn is_success(&self) -> bool {
        self.docs_failed == 0
    }
}

// ─── Raw document input ───────────────────────────────────────────────────

/// A raw document fetched from a provider, ready to be swept.
/// The caller is responsible for fetching the raw content from the API;
/// CognitiveDust only performs conversion and storage.
#[derive(Debug, Clone)]
pub struct RawProviderDocument {
    /// Provider-assigned unique identifier.
    pub source_id: String,
    /// Document title as returned by the provider.
    pub title: String,
    /// Raw content (HTML or plain text) as returned by the API export endpoint.
    pub raw_content: String,
    /// Format of the raw content.
    pub format: ProviderFormat,
    /// ISO 8601 timestamp of when the document was last modified on the provider.
    pub last_modified: String,
    /// Extra metadata from the provider API response.
    pub metadata: BTreeMap<String, String>,
}

impl RawProviderDocument {
    pub fn new(
        source_id: impl Into<String>,
        title: impl Into<String>,
        raw_content: impl Into<String>,
        format: ProviderFormat,
        last_modified: impl Into<String>,
    ) -> Self {
        RawProviderDocument {
            source_id: source_id.into(),
            title: title.into(),
            raw_content: raw_content.into(),
            format,
            last_modified: last_modified.into(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

// ─── DustSweeper ─────────────────────────────────────────────────────────

/// The CognitiveDust sweep engine.
///
/// Usage pattern:
/// ```text
/// let sweeper = DustSweeper::new();
/// let docs = fetch_from_google_drive_api(...);   // caller's responsibility
/// let result = sweeper.sweep_batch(&docs, &config, &mut manifest, &mut noosphere, "2026-03-22T10:00:00Z");
/// ```
pub struct DustSweeper;

impl DustSweeper {
    pub fn new() -> Self {
        DustSweeper
    }

    /// Process a batch of raw provider documents.
    ///
    /// For each document:
    ///   1. Check if already swept (skip if `since` filter applies).
    ///   2. Select the appropriate `universal_io` connector.
    ///   3. Call `repatriate()` to get a `UniversalDocument`.
    ///   4. Write the result to the `Noosphere` (unless dry_run).
    ///   5. Record in the `SweepManifest`.
    pub fn sweep_batch(
        &self,
        docs: &[RawProviderDocument],
        config: &SweepConfig,
        manifest: &mut SweepManifest,
        noosphere: &mut Noosphere,
        sweep_timestamp: &str,
    ) -> SweepResult {
        let provider = match &config.target {
            SweepTarget::All => "mixed",
            t => t.as_str(),
        };
        let mut result = SweepResult::new(provider, config.dry_run);
        result.docs_found = docs.len();

        let mut processed = 0;
        for doc in docs {
            // Honour the per-run limit.
            if let Some(limit) = config.limit {
                if processed >= limit {
                    result.docs_skipped += docs.len() - processed;
                    break;
                }
            }

            // Skip documents already in the manifest.
            // INVARIANT: `doc.format.provider_name()` must return the same string that
            // the `universal_io` connector stores in `frontmatter["provider"]`, which is
            // also what `manifest.record()` receives as its `provider` argument via
            // `universal_doc.provider()`.  Both call paths use `ProviderFormat::provider_name()`
            // so they remain in sync as long as connectors don't override the provider field.
            if manifest.already_swept(doc.format.provider_name(), &doc.source_id) {
                result.docs_skipped += 1;
                processed += 1;
                continue;
            }

            // Apply the `since` filter.
            if let Some(ref since) = config.since {
                if doc.last_modified < *since {
                    result.docs_skipped += 1;
                    processed += 1;
                    continue;
                }
            }

            // Convert via the appropriate connector.
            let universal_doc = match self.convert_doc(doc, sweep_timestamp) {
                Ok(d) => d,
                Err(e) => {
                    result.docs_failed += 1;
                    result.errors.push(format!("{}: {}", doc.source_id, e));
                    processed += 1;
                    continue;
                }
            };

            // Build a NoosphereEntry from the UniversalDocument.
            let noosphere_id = format!(
                "{}:{}",
                universal_doc.frontmatter.get("provider").map(|s| s.as_str()).unwrap_or("unknown"),
                doc.source_id
            );

            let resource_type = infer_resource_type(&doc.format);

            let mut entry = NoosphereEntry::new(
                &noosphere_id,
                universal_doc.title(),
                &universal_doc.content_markdown,
                universal_doc.provider(),
                resource_type,
                &doc.source_id,
                &doc.last_modified,
                sweep_timestamp,
            );

            // Apply config tags.
            for tag in &config.tags {
                entry.tags.insert(tag.clone());
            }

            // Copy extra metadata from the raw doc.
            for (k, v) in &doc.metadata {
                entry.metadata.insert(k.clone(), v.clone());
            }

            // Write to Noosphere (unless dry run).
            if !config.dry_run {
                if let Err(e) = noosphere.upsert(entry) {
                    result.docs_failed += 1;
                    result.errors.push(format!("{}: {}", doc.source_id, e));
                    processed += 1;
                    continue;
                }
            }

            // Record in manifest.
            manifest.record(
                &noosphere_id,
                universal_doc.provider(),
                &doc.source_id,
                sweep_timestamp,
                config.delete_remote,
            );

            if config.delete_remote {
                result.remote_deleted += 1;
            }

            result.docs_ingested += 1;
            processed += 1;
        }

        result
    }

    /// Convert a single raw document to a `UniversalDocument` using the
    /// appropriate `universal_io` connector.
    fn convert_doc(
        &self,
        doc: &RawProviderDocument,
        sweep_timestamp: &str,
    ) -> Result<crate::universal_io::UniversalDocument, String> {
        match &doc.format {
            ProviderFormat::GoogleDoc | ProviderFormat::GoogleSheet => {
                let connector = GoogleDocConnector::new(&doc.title);
                connector
                    .repatriate(&doc.raw_content, &doc.source_id, sweep_timestamp)
                    .map_err(|e| e.to_string())
            }
            ProviderFormat::MicrosoftWord | ProviderFormat::MicrosoftExcel => {
                let connector = MicrosoftWordConnector::new(&doc.title);
                connector
                    .repatriate(&doc.raw_content, &doc.source_id, sweep_timestamp)
                    .map_err(|e| e.to_string())
            }
            ProviderFormat::AppleNote | ProviderFormat::ApplePages => {
                let connector = AppleNoteConnector::new(&doc.title);
                connector
                    .repatriate(&doc.raw_content, &doc.source_id, sweep_timestamp)
                    .map_err(|e| e.to_string())
            }
            ProviderFormat::Markdown => {
                let connector = PlainTextConnector::markdown(&doc.title);
                connector
                    .repatriate(&doc.raw_content, &doc.source_id, sweep_timestamp)
                    .map_err(|e| e.to_string())
            }
            ProviderFormat::PlainText => {
                let connector = PlainTextConnector::plain(&doc.title);
                connector
                    .repatriate(&doc.raw_content, &doc.source_id, sweep_timestamp)
                    .map_err(|e| e.to_string())
            }
        }
    }
}

impl Default for DustSweeper {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn infer_resource_type(format: &ProviderFormat) -> ResourceType {
    match format {
        ProviderFormat::GoogleDoc
        | ProviderFormat::MicrosoftWord
        | ProviderFormat::ApplePages
        | ProviderFormat::Markdown
        | ProviderFormat::PlainText => ResourceType::Document,
        ProviderFormat::GoogleSheet | ProviderFormat::MicrosoftExcel => ResourceType::File,
        ProviderFormat::AppleNote => ResourceType::Note,
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TS: &str = "2026-03-22T10:00:00Z";

    fn google_doc(id: &str, title: &str, content: &str) -> RawProviderDocument {
        RawProviderDocument::new(id, title, content, ProviderFormat::GoogleDoc, TS)
    }

    fn word_doc(id: &str, title: &str, content: &str) -> RawProviderDocument {
        RawProviderDocument::new(id, title, content, ProviderFormat::MicrosoftWord, TS)
    }

    fn apple_note(id: &str, title: &str, content: &str) -> RawProviderDocument {
        RawProviderDocument::new(id, title, content, ProviderFormat::AppleNote, TS)
    }

    // ── Basic sweep ───────────────────────────────────────────────────

    #[test]
    fn test_sweep_single_google_doc() {
        let sweeper = DustSweeper::new();
        let docs = vec![google_doc("file_abc", "Q1 Report", "Revenue was $100K this quarter.")];
        let config = SweepConfig::new(SweepTarget::Google);
        let mut manifest = SweepManifest::new();
        let mut noosphere = Noosphere::new();

        let result = sweeper.sweep_batch(&docs, &config, &mut manifest, &mut noosphere, TS);

        assert!(result.is_success());
        assert_eq!(result.docs_ingested, 1);
        assert_eq!(result.docs_failed, 0);
        assert_eq!(noosphere.len(), 1);
        assert_eq!(manifest.len(), 1);
    }

    #[test]
    fn test_sweep_multiple_providers() {
        let sweeper = DustSweeper::new();
        let docs = vec![
            google_doc("g1", "Google Doc", "content"),
            word_doc("m1", "Word Doc", "content"),
            apple_note("a1", "Apple Note", "content"),
        ];
        let config = SweepConfig::new(SweepTarget::All);
        let mut manifest = SweepManifest::new();
        let mut noosphere = Noosphere::new();

        let result = sweeper.sweep_batch(&docs, &config, &mut manifest, &mut noosphere, TS);

        assert_eq!(result.docs_ingested, 3);
        assert_eq!(noosphere.len(), 3);
    }

    #[test]
    fn test_sweep_incremental_skip_already_swept() {
        let sweeper = DustSweeper::new();
        let docs = vec![google_doc("file_abc", "Doc", "content")];
        let config = SweepConfig::new(SweepTarget::Google);
        let mut manifest = SweepManifest::new();
        let mut noosphere = Noosphere::new();

        // First sweep.
        sweeper.sweep_batch(&docs, &config, &mut manifest, &mut noosphere, TS);

        // Second sweep — same doc should be skipped.
        let result2 = sweeper.sweep_batch(&docs, &config, &mut manifest, &mut noosphere, TS);
        assert_eq!(result2.docs_skipped, 1);
        assert_eq!(result2.docs_ingested, 0);
        assert_eq!(noosphere.len(), 1); // still only 1 entry
    }

    #[test]
    fn test_sweep_since_filter() {
        let sweeper = DustSweeper::new();
        let old_doc =
            RawProviderDocument::new("old", "Old", "content", ProviderFormat::GoogleDoc, "2026-01-01T00:00:00Z");
        let new_doc =
            RawProviderDocument::new("new", "New", "content", ProviderFormat::GoogleDoc, "2026-03-20T00:00:00Z");
        let docs = vec![old_doc, new_doc];

        let config = SweepConfig::new(SweepTarget::Google).with_since("2026-02-01T00:00:00Z");
        let mut manifest = SweepManifest::new();
        let mut noosphere = Noosphere::new();

        let result = sweeper.sweep_batch(&docs, &config, &mut manifest, &mut noosphere, TS);
        assert_eq!(result.docs_ingested, 1);
        assert_eq!(result.docs_skipped, 1);
        assert!(noosphere.get("google:new").is_some());
    }

    #[test]
    fn test_sweep_limit() {
        let sweeper = DustSweeper::new();
        let docs: Vec<RawProviderDocument> = (0..10)
            .map(|i| google_doc(&format!("id{}", i), "Doc", "content"))
            .collect();
        let config = SweepConfig::new(SweepTarget::Google).with_limit(3);
        let mut manifest = SweepManifest::new();
        let mut noosphere = Noosphere::new();

        let result = sweeper.sweep_batch(&docs, &config, &mut manifest, &mut noosphere, TS);
        assert_eq!(result.docs_ingested, 3);
        assert_eq!(noosphere.len(), 3);
    }

    #[test]
    fn test_sweep_dry_run_does_not_write() {
        let sweeper = DustSweeper::new();
        let docs = vec![google_doc("file_abc", "Doc", "content")];
        let config = SweepConfig::new(SweepTarget::Google).dry_run();
        let mut manifest = SweepManifest::new();
        let mut noosphere = Noosphere::new();

        let result = sweeper.sweep_batch(&docs, &config, &mut manifest, &mut noosphere, TS);

        assert_eq!(result.docs_ingested, 1);
        assert_eq!(noosphere.len(), 0); // dry run: nothing written
    }

    #[test]
    fn test_sweep_tags_applied() {
        let sweeper = DustSweeper::new();
        let docs = vec![google_doc("f1", "Doc", "content")];
        let config = SweepConfig::new(SweepTarget::Google)
            .with_tag("sweep-batch-1")
            .with_tag("auto-ingested");
        let mut manifest = SweepManifest::new();
        let mut noosphere = Noosphere::new();

        sweeper.sweep_batch(&docs, &config, &mut manifest, &mut noosphere, TS);

        let entry = noosphere.get("google:f1").unwrap();
        assert!(entry.tags.contains("sweep-batch-1"));
        assert!(entry.tags.contains("auto-ingested"));
    }

    #[test]
    fn test_sweep_empty_content_counts_as_failure() {
        let sweeper = DustSweeper::new();
        let docs = vec![google_doc("f1", "Empty", "   ")];
        let config = SweepConfig::new(SweepTarget::Google);
        let mut manifest = SweepManifest::new();
        let mut noosphere = Noosphere::new();

        let result = sweeper.sweep_batch(&docs, &config, &mut manifest, &mut noosphere, TS);
        assert_eq!(result.docs_failed, 1);
        assert!(!result.is_success());
    }

    #[test]
    fn test_sweep_html_content_converted() {
        let sweeper = DustSweeper::new();
        let docs = vec![RawProviderDocument::new(
            "h1",
            "HTML Doc",
            "<h1>Hello</h1><p>World</p>",
            ProviderFormat::GoogleDoc,
            TS,
        )];
        let config = SweepConfig::new(SweepTarget::Google);
        let mut manifest = SweepManifest::new();
        let mut noosphere = Noosphere::new();

        sweeper.sweep_batch(&docs, &config, &mut manifest, &mut noosphere, TS);

        let entry = noosphere.get("google:h1").unwrap();
        assert!(entry.content.contains("# Hello"));
        assert!(entry.content.contains("World"));
        assert!(!entry.content.contains("<h1>"));
    }

    #[test]
    fn test_manifest_by_provider() {
        let mut manifest = SweepManifest::new();
        manifest.record("google:g1", "google", "g1", TS, false);
        manifest.record("microsoft:m1", "microsoft", "m1", TS, false);
        manifest.record("google:g2", "google", "g2", TS, false);

        let google_entries = manifest.by_provider("google");
        assert_eq!(google_entries.len(), 2);
    }

    #[test]
    fn test_manifest_already_swept() {
        let mut manifest = SweepManifest::new();
        manifest.record("google:g1", "google", "g1", TS, false);
        assert!(manifest.already_swept("google", "g1"));
        assert!(!manifest.already_swept("google", "g2"));
        assert!(!manifest.already_swept("microsoft", "g1"));
    }
}
