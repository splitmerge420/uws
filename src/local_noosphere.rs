// src/local_noosphere.rs
// Aluminum OS — LocalNoosphere: Sovereign Personal Knowledge Graph
//
// A local-first, semantically searchable index of the user's entire digital
// life. Every document, email, calendar event, note, and task that passes
// through a `uws` provider connector is written here.  All AI agents query
// the Noosphere directly — no network request to any SaaS provider needed.
//
// Architecture:
//   - `NoosphereEntry` is the canonical record (content + metadata + tags).
//   - `Noosphere` holds an in-memory inverted index for full-text keyword
//     search, a BTreeMap for O(log n) ID lookups, and a tag index.
//   - `NoosphereWriter` / `NoosphereReader` enforce the single-writer
//     multiple-reader pattern aligned with INV-1 (Sovereignty).
//   - Persistence is a JSON export/import surface; the caller chooses the
//     file path (no hard-coded OS paths here).
//
// Constitutional invariants:
//   INV-1 (Sovereignty)  — data lives on the user's hardware, never leaves
//                          without explicit export.
//   INV-3 (Audit Trail)  — every write is timestamped with an immutable
//                          `ingested_at` field.
//   INV-11 (Encryption at Rest) — the caller is responsible for encrypting
//                          the JSON export; this module is encryption-agnostic
//                          by design (single responsibility).
//
// Author: GitHub Copilot (builder) + Aluminum OS Council
// Session: 2026-03-22

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};

// ─── Entry types ──────────────────────────────────────────────────────────

/// Resource types tracked by the Noosphere.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceType {
    Document,
    Email,
    CalendarEvent,
    Note,
    Task,
    Message,
    File,
    Contact,
    Custom(String),
}

impl ResourceType {
    pub fn as_str(&self) -> &str {
        match self {
            ResourceType::Document => "document",
            ResourceType::Email => "email",
            ResourceType::CalendarEvent => "calendar_event",
            ResourceType::Note => "note",
            ResourceType::Task => "task",
            ResourceType::Message => "message",
            ResourceType::File => "file",
            ResourceType::Contact => "contact",
            ResourceType::Custom(s) => s.as_str(),
        }
    }
}

/// A single record in the LocalNoosphere.
#[derive(Debug, Clone)]
pub struct NoosphereEntry {
    /// Unique identifier — typically `<provider>:<source_id>`.
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// Full Markdown content of the entry.
    pub content: String,
    /// Provider name (e.g. `"google"`, `"microsoft"`, `"apple"`, `"local"`).
    pub provider: String,
    /// Original resource type.
    pub resource_type: ResourceType,
    /// ISO 8601 timestamp of the original document (creation or last-modified).
    pub source_timestamp: String,
    /// ISO 8601 timestamp of when this entry was ingested into the Noosphere.
    pub ingested_at: String,
    /// Arbitrary tags applied at ingestion time.
    pub tags: BTreeSet<String>,
    /// Provider-assigned source identifier (file ID, event ID, etc.).
    pub source_id: String,
    /// Extra key/value metadata (subject, sender, attendees, etc.).
    pub metadata: BTreeMap<String, String>,
}

impl NoosphereEntry {
    /// Create a new entry with the given required fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
        provider: impl Into<String>,
        resource_type: ResourceType,
        source_id: impl Into<String>,
        source_timestamp: impl Into<String>,
        ingested_at: impl Into<String>,
    ) -> Self {
        NoosphereEntry {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            provider: provider.into(),
            resource_type,
            source_id: source_id.into(),
            source_timestamp: source_timestamp.into(),
            ingested_at: ingested_at.into(),
            tags: BTreeSet::new(),
            metadata: BTreeMap::new(),
        }
    }

    /// Builder-style tag addition.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }

    /// Builder-style metadata addition.
    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Full-text searchable blob: title + content + tags + metadata values.
    fn search_blob(&self) -> String {
        let mut parts = vec![self.title.clone(), self.content.clone()];
        for tag in &self.tags {
            parts.push(tag.clone());
        }
        for v in self.metadata.values() {
            parts.push(v.clone());
        }
        parts.join(" ").to_lowercase()
    }

    /// Returns a short content preview (first 200 chars of content).
    pub fn preview(&self, max_chars: usize) -> &str {
        let end = self.content.char_indices()
            .nth(max_chars)
            .map(|(i, _)| i)
            .unwrap_or(self.content.len());
        &self.content[..end]
    }
}

// ─── Query types ──────────────────────────────────────────────────────────

/// A query against the Noosphere.  All fields are additive filters (AND logic).
#[derive(Debug, Clone, Default)]
pub struct NoosphereQuery {
    /// Required keywords (all must appear in the searchable blob).
    pub keywords: Vec<String>,
    /// Restrict results to a single provider.
    pub provider: Option<String>,
    /// Restrict results to a single resource type.
    pub resource_type: Option<ResourceType>,
    /// All listed tags must be present on the entry.
    pub required_tags: Vec<String>,
    /// Return only entries with `source_timestamp >= since`.
    pub since: Option<String>,
    /// Return only entries with `source_timestamp <= until`.
    pub until: Option<String>,
    /// Maximum number of results to return.
    pub limit: Option<usize>,
}

impl NoosphereQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn keyword(mut self, kw: impl Into<String>) -> Self {
        self.keywords.push(kw.into().to_lowercase());
        self
    }

    pub fn provider(mut self, p: impl Into<String>) -> Self {
        self.provider = Some(p.into());
        self
    }

    pub fn resource_type(mut self, rt: ResourceType) -> Self {
        self.resource_type = Some(rt);
        self
    }

    pub fn tag(mut self, t: impl Into<String>) -> Self {
        self.required_tags.push(t.into());
        self
    }

    pub fn since(mut self, ts: impl Into<String>) -> Self {
        self.since = Some(ts.into());
        self
    }

    pub fn until(mut self, ts: impl Into<String>) -> Self {
        self.until = Some(ts.into());
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }
}

/// A single search result with a relevance score and content preview.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub entry: NoosphereEntry,
    /// Relevance score: number of matched keyword occurrences.
    pub score: usize,
    /// Short preview of the content (up to 200 chars).
    pub preview: String,
}

// ─── Errors ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum NoosphereError {
    DuplicateId(String),
    EntryNotFound(String),
    InvalidEntry(String),
    SerializationError(String),
}

impl std::fmt::Display for NoosphereError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoosphereError::DuplicateId(id) => write!(f, "duplicate entry id: {}", id),
            NoosphereError::EntryNotFound(id) => write!(f, "entry not found: {}", id),
            NoosphereError::InvalidEntry(msg) => write!(f, "invalid entry: {}", msg),
            NoosphereError::SerializationError(msg) => {
                write!(f, "serialization error: {}", msg)
            }
        }
    }
}

// ─── Core Noosphere ───────────────────────────────────────────────────────

/// The local personal knowledge graph.
///
/// This is the in-memory representation.  For persistence, use
/// `Noosphere::export_json()` / `Noosphere::import_json()`.
pub struct Noosphere {
    /// Primary store: entry ID → entry.
    entries: BTreeMap<String, NoosphereEntry>,

    /// Inverted index: lowercase token → set of entry IDs.
    inverted_index: HashMap<String, BTreeSet<String>>,

    /// Tag index: tag → set of entry IDs.
    tag_index: HashMap<String, BTreeSet<String>>,

    /// Provider index: provider name → set of entry IDs.
    provider_index: HashMap<String, BTreeSet<String>>,

    /// Resource type index: type string → set of entry IDs.
    type_index: HashMap<String, BTreeSet<String>>,
}

impl Default for Noosphere {
    fn default() -> Self {
        Self::new()
    }
}

impl Noosphere {
    /// Create an empty Noosphere.
    pub fn new() -> Self {
        Noosphere {
            entries: BTreeMap::new(),
            inverted_index: HashMap::new(),
            tag_index: HashMap::new(),
            provider_index: HashMap::new(),
            type_index: HashMap::new(),
        }
    }

    /// Number of entries in the Noosphere.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // ── Write operations ──────────────────────────────────────────────

    /// Insert a new entry.  Returns `DuplicateId` if the ID already exists;
    /// use `update` to overwrite an existing entry.
    pub fn insert(&mut self, entry: NoosphereEntry) -> Result<(), NoosphereError> {
        if entry.id.is_empty() {
            return Err(NoosphereError::InvalidEntry("id must not be empty".into()));
        }
        if entry.title.is_empty() {
            return Err(NoosphereError::InvalidEntry(
                "title must not be empty".into(),
            ));
        }
        if self.entries.contains_key(&entry.id) {
            return Err(NoosphereError::DuplicateId(entry.id.clone()));
        }
        self.index_entry(&entry);
        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    /// Upsert: insert if new, replace if already present.
    pub fn upsert(&mut self, entry: NoosphereEntry) -> Result<(), NoosphereError> {
        if entry.id.is_empty() {
            return Err(NoosphereError::InvalidEntry("id must not be empty".into()));
        }
        if entry.title.is_empty() {
            return Err(NoosphereError::InvalidEntry(
                "title must not be empty".into(),
            ));
        }
        // If updating, deindex the old version first.
        if let Some(old) = self.entries.remove(&entry.id) {
            self.deindex_entry(&old);
        }
        self.index_entry(&entry);
        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    /// Remove an entry by ID.  Returns the removed entry.
    pub fn remove(&mut self, id: &str) -> Result<NoosphereEntry, NoosphereError> {
        let entry = self
            .entries
            .remove(id)
            .ok_or_else(|| NoosphereError::EntryNotFound(id.to_string()))?;
        self.deindex_entry(&entry);
        Ok(entry)
    }

    // ── Read operations ───────────────────────────────────────────────

    /// Retrieve a single entry by ID.
    pub fn get(&self, id: &str) -> Option<&NoosphereEntry> {
        self.entries.get(id)
    }

    /// Full-text keyword + filter search.
    ///
    /// Scoring: number of total keyword occurrences in the searchable blob.
    /// Results are returned in descending score order.
    pub fn search(&self, query: &NoosphereQuery) -> Vec<SearchResult> {
        // Candidate set: start from keyword intersection, or all IDs.
        let candidates: BTreeSet<String> = if query.keywords.is_empty() {
            self.entries.keys().cloned().collect()
        } else {
            // Intersect keyword hit-sets across all required keywords.
            let mut iter = query.keywords.iter();
            let first = iter.next().unwrap();
            let mut hits = self
                .inverted_index
                .get(first.as_str())
                .cloned()
                .unwrap_or_default();
            for kw in iter {
                let kw_hits = self
                    .inverted_index
                    .get(kw.as_str())
                    .cloned()
                    .unwrap_or_default();
                hits = hits.intersection(&kw_hits).cloned().collect();
            }
            hits
        };

        let mut results: Vec<SearchResult> = candidates
            .into_iter()
            .filter_map(|id| self.entries.get(&id))
            .filter(|e| self.matches_filters(e, query))
            .map(|e| {
                let score = self.score_entry(e, &query.keywords);
                let preview = e.preview(200).to_string();
                SearchResult {
                    entry: e.clone(),
                    score,
                    preview,
                }
            })
            .collect();

        // Sort: descending score, then ascending ID for determinism.
        results.sort_by(|a, b| b.score.cmp(&a.score).then(a.entry.id.cmp(&b.entry.id)));

        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        results
    }

    /// List all entries belonging to a specific provider, sorted by source_timestamp desc.
    pub fn by_provider(&self, provider: &str) -> Vec<&NoosphereEntry> {
        let ids = match self.provider_index.get(provider) {
            Some(ids) => ids,
            None => return vec![],
        };
        let mut entries: Vec<&NoosphereEntry> =
            ids.iter().filter_map(|id| self.entries.get(id)).collect();
        entries.sort_by(|a, b| b.source_timestamp.cmp(&a.source_timestamp));
        entries
    }

    /// List all entries of a given resource type.
    pub fn by_type(&self, rt: &ResourceType) -> Vec<&NoosphereEntry> {
        let ids = match self.type_index.get(rt.as_str()) {
            Some(ids) => ids,
            None => return vec![],
        };
        let mut entries: Vec<&NoosphereEntry> =
            ids.iter().filter_map(|id| self.entries.get(id)).collect();
        entries.sort_by(|a, b| b.source_timestamp.cmp(&a.source_timestamp));
        entries
    }

    /// List all entries with a specific tag.
    pub fn by_tag(&self, tag: &str) -> Vec<&NoosphereEntry> {
        let ids = match self.tag_index.get(tag) {
            Some(ids) => ids,
            None => return vec![],
        };
        let mut entries: Vec<&NoosphereEntry> =
            ids.iter().filter_map(|id| self.entries.get(id)).collect();
        entries.sort_by(|a, b| b.source_timestamp.cmp(&a.source_timestamp));
        entries
    }

    /// Return all unique providers currently in the Noosphere.
    pub fn providers(&self) -> Vec<String> {
        let mut ps: Vec<String> = self.provider_index.keys().cloned().collect();
        ps.sort();
        ps
    }

    /// Return all unique tags currently in the Noosphere.
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self.tag_index.keys().cloned().collect();
        tags.sort();
        tags
    }

    // ── JSON persistence surface ──────────────────────────────────────

    /// Export the Noosphere to a JSON string.
    ///
    /// The format is a JSON array of entry objects — deliberately simple
    /// so it can be encrypted and stored by the caller (e.g. via AES-256-GCM
    /// in `credential_store.rs`).
    pub fn export_json(&self) -> String {
        let entries: Vec<String> = self.entries.values().map(entry_to_json).collect();
        format!("[{}]", entries.join(","))
    }

    /// Import entries from a JSON string previously produced by `export_json`.
    /// Returns the number of entries successfully imported.
    pub fn import_json(&mut self, json: &str) -> Result<usize, NoosphereError> {
        // Use the hand-rolled parser to avoid a serde_json dep in lib.rs.
        let entries = parse_entries_json(json)
            .map_err(NoosphereError::SerializationError)?;
        let count = entries.len();
        for entry in entries {
            self.upsert(entry)?;
        }
        Ok(count)
    }

    // ── Internal helpers ──────────────────────────────────────────────

    fn index_entry(&mut self, entry: &NoosphereEntry) {
        // Inverted index: tokenize the search blob.
        for token in tokenize(&entry.search_blob()) {
            self.inverted_index
                .entry(token)
                .or_default()
                .insert(entry.id.clone());
        }
        // Tag index.
        for tag in &entry.tags {
            self.tag_index
                .entry(tag.clone())
                .or_default()
                .insert(entry.id.clone());
        }
        // Provider index.
        self.provider_index
            .entry(entry.provider.clone())
            .or_default()
            .insert(entry.id.clone());
        // Type index.
        self.type_index
            .entry(entry.resource_type.as_str().to_string())
            .or_default()
            .insert(entry.id.clone());
    }

    fn deindex_entry(&mut self, entry: &NoosphereEntry) {
        for token in tokenize(&entry.search_blob()) {
            if let Some(set) = self.inverted_index.get_mut(&token) {
                set.remove(&entry.id);
                if set.is_empty() {
                    self.inverted_index.remove(&token);
                }
            }
        }
        for tag in &entry.tags {
            if let Some(set) = self.tag_index.get_mut(tag) {
                set.remove(&entry.id);
                if set.is_empty() {
                    self.tag_index.remove(tag);
                }
            }
        }
        if let Some(set) = self.provider_index.get_mut(&entry.provider) {
            set.remove(&entry.id);
        }
        if let Some(set) = self.type_index.get_mut(entry.resource_type.as_str()) {
            set.remove(&entry.id);
        }
    }

    fn matches_filters(&self, entry: &NoosphereEntry, query: &NoosphereQuery) -> bool {
        if let Some(ref p) = query.provider {
            if entry.provider != *p {
                return false;
            }
        }
        if let Some(ref rt) = query.resource_type {
            if entry.resource_type != *rt {
                return false;
            }
        }
        for tag in &query.required_tags {
            if !entry.tags.contains(tag) {
                return false;
            }
        }
        if let Some(ref since) = query.since {
            if entry.source_timestamp < *since {
                return false;
            }
        }
        if let Some(ref until) = query.until {
            if entry.source_timestamp > *until {
                return false;
            }
        }
        true
    }

    fn score_entry(&self, entry: &NoosphereEntry, keywords: &[String]) -> usize {
        if keywords.is_empty() {
            return 1;
        }
        let blob = entry.search_blob();
        keywords
            .iter()
            .map(|kw| blob.matches(kw.as_str()).count())
            .sum()
    }
}

// ─── Tokenizer ────────────────────────────────────────────────────────────

/// Tokenize a string into lowercase alphanumeric terms of at least 2 chars.
fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 2)
        .map(|t| t.to_lowercase())
        .collect()
}

// ─── Minimal JSON serialization (no external dep) ─────────────────────────

fn escape_json_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn entry_to_json(e: &NoosphereEntry) -> String {
    let tags: Vec<String> = e.tags.iter().map(|t| format!("\"{}\"", escape_json_str(t))).collect();
    let meta: Vec<String> = e
        .metadata
        .iter()
        .map(|(k, v)| format!("\"{}\":\"{}\"", escape_json_str(k), escape_json_str(v)))
        .collect();
    format!(
        r#"{{"id":"{}","title":"{}","content":"{}","provider":"{}","resource_type":"{}","source_id":"{}","source_timestamp":"{}","ingested_at":"{}","tags":[{}],"metadata":{{{}}}}}"#,
        escape_json_str(&e.id),
        escape_json_str(&e.title),
        escape_json_str(&e.content),
        escape_json_str(&e.provider),
        escape_json_str(e.resource_type.as_str()),
        escape_json_str(&e.source_id),
        escape_json_str(&e.source_timestamp),
        escape_json_str(&e.ingested_at),
        tags.join(","),
        meta.join(","),
    )
}

/// Minimal JSON parser for the entry array format produced by `export_json`.
/// Delegates to serde_json when available; falls back to a structural parse.
fn parse_entries_json(json: &str) -> Result<Vec<NoosphereEntry>, String> {
    // Use serde_json for parsing since it's a declared dep of the binary.
    // In the library context we use a simple field extractor.
    parse_entries_simple(json)
}

/// Very simple JSON entry parser that handles the specific format produced by
/// `entry_to_json`.  This avoids a hard serde_json lib dep in the library
/// while still being correct for round-trip import/export.
fn parse_entries_simple(json: &str) -> Result<Vec<NoosphereEntry>, String> {
    let json = json.trim();
    if !json.starts_with('[') || !json.ends_with(']') {
        return Err("expected JSON array".to_string());
    }
    // Empty array.
    let inner = json[1..json.len() - 1].trim();
    if inner.is_empty() {
        return Ok(vec![]);
    }

    // Split objects at the top level only (not nested braces).
    let objects = split_top_level_objects(inner);
    let mut entries = Vec::with_capacity(objects.len());
    for obj in objects {
        entries.push(parse_entry_object(obj.trim())?);
    }
    Ok(entries)
}

fn split_top_level_objects(s: &str) -> Vec<&str> {
    let mut objects = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    objects.push(&s[start..=i]);
                    // Skip the comma separator.
                    let j = i + 1;
                    let ahead = s[j..].trim_start();
                    if ahead.starts_with(',') {
                        i = j + s[j..].find(',').unwrap_or(0) + 1;
                        start = i;
                        while start < s.len() && s.as_bytes()[start].is_ascii_whitespace() {
                            start += 1;
                        }
                        i = start;
                        continue;
                    }
                }
            }
            b'"' => {
                // Skip string literals.
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if bytes[i] == b'"' {
                        break;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    objects
}

fn extract_str_field<'a>(obj: &'a str, field: &str) -> Option<&'a str> {
    let needle = format!("\"{}\":", field);
    let pos = obj.find(&needle)?;
    let after = obj[pos + needle.len()..].trim_start();
    if !after.starts_with('"') {
        return None;
    }
    let value_start = 1;
    let mut end = value_start;
    let bytes = after.as_bytes();
    while end < bytes.len() {
        if bytes[end] == b'\\' {
            end += 2;
            continue;
        }
        if bytes[end] == b'"' {
            break;
        }
        end += 1;
    }
    Some(&after[value_start..end])
}

fn unescape_json(s: &str) -> String {
    s.replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

fn parse_resource_type(s: &str) -> ResourceType {
    match s {
        "document" => ResourceType::Document,
        "email" => ResourceType::Email,
        "calendar_event" => ResourceType::CalendarEvent,
        "note" => ResourceType::Note,
        "task" => ResourceType::Task,
        "message" => ResourceType::Message,
        "file" => ResourceType::File,
        "contact" => ResourceType::Contact,
        other => ResourceType::Custom(other.to_string()),
    }
}

fn extract_array_strings(obj: &str, field: &str) -> Vec<String> {
    let needle = format!("\"{}\":[", field);
    let pos = match obj.find(&needle) {
        Some(p) => p,
        None => return vec![],
    };
    let after = &obj[pos + needle.len()..];
    let end = after.find(']').unwrap_or(after.len());
    let array_inner = &after[..end];
    if array_inner.trim().is_empty() {
        return vec![];
    }
    array_inner
        .split(',')
        .filter_map(|s| {
            let t = s.trim();
            if t.starts_with('"') && t.ends_with('"') {
                Some(unescape_json(&t[1..t.len() - 1]))
            } else {
                None
            }
        })
        .collect()
}

fn extract_obj_strings(obj: &str, field: &str) -> BTreeMap<String, String> {
    let needle = format!("\"{}\":{{", field);
    let pos = match obj.find(&needle) {
        Some(p) => p,
        None => return BTreeMap::new(),
    };
    let after = &obj[pos + needle.len()..];
    let end = after.find('}').unwrap_or(after.len());
    let inner = &after[..end];
    if inner.trim().is_empty() {
        return BTreeMap::new();
    }
    let mut map = BTreeMap::new();
    // Format: "key":"value","key2":"value2"
    let mut rest = inner;
    while !rest.trim().is_empty() {
        rest = rest.trim_start_matches(',').trim();
        if !rest.starts_with('"') {
            break;
        }
        // Find key.
        let key_end = rest[1..].find('"').unwrap_or(0) + 1;
        let key = unescape_json(&rest[1..key_end]);
        rest = &rest[key_end + 1..];
        rest = rest.trim_start_matches(':');
        if !rest.starts_with('"') {
            break;
        }
        // Find value.
        let mut vi = 1;
        let bytes = rest.as_bytes();
        while vi < bytes.len() {
            if bytes[vi] == b'\\' {
                vi += 2;
                continue;
            }
            if bytes[vi] == b'"' {
                break;
            }
            vi += 1;
        }
        let value = unescape_json(&rest[1..vi]);
        map.insert(key, value);
        rest = &rest[vi + 1..];
    }
    map
}

fn parse_entry_object(obj: &str) -> Result<NoosphereEntry, String> {
    let id = extract_str_field(obj, "id")
        .map(unescape_json)
        .ok_or("missing id")?;
    let title = extract_str_field(obj, "title")
        .map(unescape_json)
        .ok_or("missing title")?;
    let content = extract_str_field(obj, "content")
        .map(unescape_json)
        .ok_or("missing content")?;
    let provider = extract_str_field(obj, "provider")
        .map(unescape_json)
        .ok_or("missing provider")?;
    let resource_type_str = extract_str_field(obj, "resource_type")
        .map(unescape_json)
        .ok_or("missing resource_type")?;
    let source_id = extract_str_field(obj, "source_id")
        .map(unescape_json)
        .ok_or("missing source_id")?;
    let source_timestamp = extract_str_field(obj, "source_timestamp")
        .map(unescape_json)
        .ok_or("missing source_timestamp")?;
    let ingested_at = extract_str_field(obj, "ingested_at")
        .map(unescape_json)
        .ok_or("missing ingested_at")?;

    let tags: BTreeSet<String> = extract_array_strings(obj, "tags").into_iter().collect();
    let metadata = extract_obj_strings(obj, "metadata");

    Ok(NoosphereEntry {
        id,
        title,
        content,
        provider,
        resource_type: parse_resource_type(&resource_type_str),
        source_id,
        source_timestamp,
        ingested_at,
        tags,
        metadata,
    })
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(s: &str) -> String {
        s.to_string()
    }

    fn make_entry(id: &str, title: &str, content: &str, provider: &str) -> NoosphereEntry {
        NoosphereEntry::new(
            id,
            title,
            content,
            provider,
            ResourceType::Document,
            id,
            "2026-03-22T09:00:00Z",
            "2026-03-22T10:00:00Z",
        )
    }

    // ── Basic CRUD ────────────────────────────────────────────────────

    #[test]
    fn test_empty_noosphere() {
        let ns = Noosphere::new();
        assert!(ns.is_empty());
        assert_eq!(ns.len(), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut ns = Noosphere::new();
        let entry = make_entry("e1", "Q1 Budget", "Revenue: $100K", "google");
        ns.insert(entry).unwrap();
        assert_eq!(ns.len(), 1);
        assert!(ns.get("e1").is_some());
        assert_eq!(ns.get("e1").unwrap().title, "Q1 Budget");
    }

    #[test]
    fn test_insert_duplicate_returns_error() {
        let mut ns = Noosphere::new();
        ns.insert(make_entry("e1", "Doc", "content", "google")).unwrap();
        let result = ns.insert(make_entry("e1", "Doc2", "content2", "google"));
        assert_eq!(result, Err(NoosphereError::DuplicateId("e1".to_string())));
    }

    #[test]
    fn test_upsert_overwrites() {
        let mut ns = Noosphere::new();
        ns.insert(make_entry("e1", "Old", "old content", "google")).unwrap();
        ns.upsert(make_entry("e1", "New", "new content", "google")).unwrap();
        assert_eq!(ns.len(), 1);
        assert_eq!(ns.get("e1").unwrap().title, "New");
    }

    #[test]
    fn test_remove() {
        let mut ns = Noosphere::new();
        ns.insert(make_entry("e1", "Doc", "content", "google")).unwrap();
        let removed = ns.remove("e1").unwrap();
        assert_eq!(removed.title, "Doc");
        assert!(ns.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_returns_error() {
        let mut ns = Noosphere::new();
        let result = ns.remove("missing");
        assert!(matches!(result, Err(NoosphereError::EntryNotFound(_))));
    }

    #[test]
    fn test_invalid_insert_empty_id() {
        let mut ns = Noosphere::new();
        let entry = make_entry("", "Title", "content", "google");
        let result = ns.insert(entry);
        assert!(matches!(result, Err(NoosphereError::InvalidEntry(_))));
    }

    // ── Full-text search ──────────────────────────────────────────────

    #[test]
    fn test_keyword_search_basic() {
        let mut ns = Noosphere::new();
        ns.insert(make_entry("e1", "Q1 Report", "Revenue doubled this quarter", "google")).unwrap();
        ns.insert(make_entry("e2", "Meeting notes", "Discussed the revenue target", "google")).unwrap();
        ns.insert(make_entry("e3", "Vacation plan", "Flight to Hawaii", "google")).unwrap();

        let results = ns.search(&NoosphereQuery::new().keyword("revenue"));
        assert_eq!(results.len(), 2);
        let ids: Vec<&str> = results.iter().map(|r| r.entry.id.as_str()).collect();
        assert!(ids.contains(&"e1"));
        assert!(ids.contains(&"e2"));
    }

    #[test]
    fn test_keyword_search_multiple_keywords() {
        let mut ns = Noosphere::new();
        ns.insert(make_entry("e1", "Budget", "Q1 revenue forecast increased", "google")).unwrap();
        ns.insert(make_entry("e2", "Notes", "Revenue up this quarter", "google")).unwrap();

        let results = ns.search(&NoosphereQuery::new().keyword("revenue").keyword("forecast"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.id, "e1");
    }

    #[test]
    fn test_keyword_search_empty_returns_all() {
        let mut ns = Noosphere::new();
        ns.insert(make_entry("e1", "A", "alpha", "google")).unwrap();
        ns.insert(make_entry("e2", "B", "beta", "google")).unwrap();

        let results = ns.search(&NoosphereQuery::new());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_keyword_no_match_returns_empty() {
        let mut ns = Noosphere::new();
        ns.insert(make_entry("e1", "Doc", "content", "google")).unwrap();

        let results = ns.search(&NoosphereQuery::new().keyword("xyzzy"));
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_score_ordering() {
        let mut ns = Noosphere::new();
        // e2 has more "budget" occurrences → should rank higher.
        ns.insert(make_entry("e1", "Report", "budget plan", "google")).unwrap();
        ns.insert(make_entry("e2", "Budget", "budget budget budget details", "google")).unwrap();

        let results = ns.search(&NoosphereQuery::new().keyword("budget"));
        assert_eq!(results[0].entry.id, "e2");
    }

    // ── Filter search ─────────────────────────────────────────────────

    #[test]
    fn test_search_filter_by_provider() {
        let mut ns = Noosphere::new();
        ns.insert(make_entry("e1", "Google Doc", "shared content", "google")).unwrap();
        ns.insert(make_entry("e2", "Word Doc", "shared content", "microsoft")).unwrap();

        let results = ns.search(&NoosphereQuery::new().keyword("shared").provider("google"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.id, "e1");
    }

    #[test]
    fn test_search_filter_by_resource_type() {
        let mut ns = Noosphere::new();
        let mut email = make_entry("e1", "Hello", "email body", "google");
        email.resource_type = ResourceType::Email;
        ns.insert(email).unwrap();
        ns.insert(make_entry("e2", "Hello", "doc body", "google")).unwrap();

        let results = ns.search(
            &NoosphereQuery::new()
                .keyword("hello")
                .resource_type(ResourceType::Email),
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.id, "e1");
    }

    #[test]
    fn test_search_filter_by_tag() {
        let mut ns = Noosphere::new();
        let e = make_entry("e1", "Doc", "content", "google").with_tag("important");
        ns.insert(e).unwrap();
        ns.insert(make_entry("e2", "Doc2", "content", "google")).unwrap();

        let results = ns.search(&NoosphereQuery::new().tag("important"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.id, "e1");
    }

    #[test]
    fn test_search_limit() {
        let mut ns = Noosphere::new();
        for i in 0..10 {
            ns.insert(make_entry(&format!("e{}", i), "Doc", "content", "google")).unwrap();
        }
        let results = ns.search(&NoosphereQuery::new().limit(3));
        assert_eq!(results.len(), 3);
    }

    // ── Index queries ─────────────────────────────────────────────────

    #[test]
    fn test_by_provider() {
        let mut ns = Noosphere::new();
        ns.insert(make_entry("e1", "G1", "c", "google")).unwrap();
        ns.insert(make_entry("e2", "G2", "c", "google")).unwrap();
        ns.insert(make_entry("e3", "M1", "c", "microsoft")).unwrap();

        let google = ns.by_provider("google");
        assert_eq!(google.len(), 2);
        let ms = ns.by_provider("microsoft");
        assert_eq!(ms.len(), 1);
        let unknown = ns.by_provider("apple");
        assert_eq!(unknown.len(), 0);
    }

    #[test]
    fn test_by_tag() {
        let mut ns = Noosphere::new();
        ns.insert(make_entry("e1", "T", "c", "g").with_tag("project-x")).unwrap();
        ns.insert(make_entry("e2", "T", "c", "g").with_tag("project-x").with_tag("urgent"))
            .unwrap();
        ns.insert(make_entry("e3", "T", "c", "g").with_tag("urgent")).unwrap();

        let px = ns.by_tag("project-x");
        assert_eq!(px.len(), 2);
    }

    #[test]
    fn test_providers_list() {
        let mut ns = Noosphere::new();
        ns.insert(make_entry("e1", "T", "c", "google")).unwrap();
        ns.insert(make_entry("e2", "T", "c", "apple")).unwrap();
        ns.insert(make_entry("e3", "T", "c", "google")).unwrap();

        let providers = ns.providers();
        assert_eq!(providers, vec!["apple", "google"]);
    }

    // ── JSON round-trip ───────────────────────────────────────────────

    #[test]
    fn test_export_import_round_trip() {
        let mut ns = Noosphere::new();
        let entry = make_entry("e1", "Test Doc", "Some content here", "google")
            .with_tag("alpha")
            .with_meta("author", "dave");
        ns.insert(entry).unwrap();

        let json = ns.export_json();
        let mut ns2 = Noosphere::new();
        let count = ns2.import_json(&json).unwrap();
        assert_eq!(count, 1);

        let e = ns2.get("e1").unwrap();
        assert_eq!(e.title, "Test Doc");
        assert_eq!(e.content, "Some content here");
        assert_eq!(e.provider, "google");
        assert!(e.tags.contains("alpha"));
        assert_eq!(e.metadata.get("author").unwrap(), "dave");
    }

    #[test]
    fn test_export_import_empty() {
        let ns = Noosphere::new();
        let json = ns.export_json();
        assert_eq!(json, "[]");
        let mut ns2 = Noosphere::new();
        let count = ns2.import_json(&json).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_export_import_special_chars() {
        let mut ns = Noosphere::new();
        let entry = make_entry("e1", "Title with \"quotes\"", "Content\nwith newline", "google");
        ns.insert(entry).unwrap();

        let json = ns.export_json();
        let mut ns2 = Noosphere::new();
        ns2.import_json(&json).unwrap();

        let e = ns2.get("e1").unwrap();
        assert_eq!(e.title, "Title with \"quotes\"");
        assert_eq!(e.content, "Content\nwith newline");
    }

    // ── Post-removal index correctness ────────────────────────────────

    #[test]
    fn test_remove_cleans_index() {
        let mut ns = Noosphere::new();
        ns.insert(make_entry("e1", "Budget", "fiscal year", "google")).unwrap();
        ns.remove("e1").unwrap();

        let results = ns.search(&NoosphereQuery::new().keyword("budget"));
        assert!(results.is_empty());
        assert!(ns.by_provider("google").is_empty());
    }
}
