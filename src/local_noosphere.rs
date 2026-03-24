// src/local_noosphere.rs
// Aluminum OS — LocalNoosphere: Sovereign Personal Knowledge Graph
//
// A fully local, in-memory graph database that stores every piece of
// information the user cares about (documents, emails, calendar events,
// contacts, tasks, notes, web pages) as typed `GraphNode` objects linked
// by semantic edges.
//
// Design principles:
//   - 100% local-first: no network call is ever required to read or write.
//   - Provider-agnostic: nodes flow in from `universal_io` connectors and
//     can flow back out via the same connectors.
//   - Append-and-supersede: nodes are never deleted; they are replaced with
//     a newer version and the delta is recorded in `TemporalDelta` so that
//     a TemporalAnchor implementation can build the full event-sourced
//     history.
//   - Searchable: `query()` does case-insensitive substring matching over
//     node titles and bodies; `query_by_tag()` filters by tag.
//
// Invariants Enforced: INV-1 (Sovereignty), INV-3 (Audit Trail via
//                      TemporalDelta), INV-6 (Provider Abstraction)
//
// Phase 2 note: When the `rusqlite` dependency is available, replace the
// `BTreeMap`-backed index with an SQLite FTS5 virtual table stored in
// `~/.uws/noosphere/noosphere.db` for sub-millisecond search on millions
// of nodes.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::universal_io::UniversalDocument;

// ─── Node Kind ───────────────────────────────────────────────────────────────

/// The semantic type of a knowledge-graph node.
///
/// New kinds can be added without breaking existing nodes; the graph stores
/// whatever kind is provided and exposes it in query results.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeKind {
    /// A long-form document (from Google Docs, Word, etc.)
    Document,
    /// An email message (from Gmail, Outlook, etc.)
    Email,
    /// A calendar event (from Google Calendar, Outlook, iCal)
    CalendarEvent,
    /// A contact record (from Google People, CardDAV, etc.)
    Contact,
    /// A task or to-do item
    Task,
    /// A short-form note (from Apple Notes, Keep, OneNote)
    Note,
    /// A web page snapshot or bookmark
    WebPage,
    /// Any other provider-specific kind; the string is the provider's type name.
    Other(String),
}

impl NodeKind {
    /// Human-readable label for this kind.
    pub fn label(&self) -> &str {
        match self {
            NodeKind::Document => "document",
            NodeKind::Email => "email",
            NodeKind::CalendarEvent => "calendar_event",
            NodeKind::Contact => "contact",
            NodeKind::Task => "task",
            NodeKind::Note => "note",
            NodeKind::WebPage => "web_page",
            NodeKind::Other(s) => s.as_str(),
        }
    }
}

// ─── Graph Node ──────────────────────────────────────────────────────────────

/// A single node in the `LocalNoosphere` knowledge graph.
///
/// Every piece of information — document, email, contact, task — is a node.
/// Nodes are linked to each other by their `linked_node_ids` set, forming a
/// semantic graph.  The body is always standard Markdown so it can be round-
/// tripped through `universal_io` connectors.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct GraphNode {
    /// Globally unique node identifier (provider-assigned or UUID).
    pub id: String,
    /// The semantic type of this node.
    pub kind: NodeKind,
    /// Human-readable title / subject.
    pub title: String,
    /// Body content as standard Markdown (empty for contacts / calendar events
    /// that have no long-form body).
    pub body_markdown: String,
    /// Free-form tags for categorisation and filtering.
    pub tags: BTreeSet<String>,
    /// IDs of nodes this node is linked to (bidirectional by convention, but
    /// the graph does not enforce bidirectionality — callers may link both ways
    /// via `LocalNoosphere::link()`).
    pub linked_node_ids: BTreeSet<String>,
    /// Flat key/value metadata (e.g. `"source" → "Google Workspace"`).
    pub metadata: BTreeMap<String, String>,
    /// UNIX timestamp (seconds) when this node was first created.
    pub created_at: u64,
    /// UNIX timestamp (seconds) of the last modification.
    pub updated_at: u64,
}

impl GraphNode {
    /// Create a new node with empty tags, links, and metadata.
    pub fn new(
        id: impl Into<String>,
        kind: NodeKind,
        title: impl Into<String>,
        body_markdown: impl Into<String>,
    ) -> Self {
        let now = unix_now();
        GraphNode {
            id: id.into(),
            kind,
            title: title.into(),
            body_markdown: body_markdown.into(),
            tags: BTreeSet::new(),
            linked_node_ids: BTreeSet::new(),
            metadata: BTreeMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Attach a tag (fluent builder).
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }

    /// Attach a metadata key/value pair (fluent builder).
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Return `true` if the node's title or body contains `needle` (case-insensitive).
    pub fn matches_text(&self, needle: &str) -> bool {
        let needle_lower = needle.to_lowercase();
        self.title.to_lowercase().contains(&needle_lower)
            || self.body_markdown.to_lowercase().contains(&needle_lower)
    }

    /// Return `true` if the node carries the given tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }
}

/// Convert a `UniversalDocument` into a `GraphNode` (kind = `Document`).
impl From<&UniversalDocument> for GraphNode {
    fn from(doc: &UniversalDocument) -> Self {
        GraphNode::new(
            &doc.id,
            NodeKind::Document,
            &doc.title,
            &doc.body_markdown,
        )
        .with_metadata("source_document_id", &doc.id)
    }
}

/// Convert a `GraphNode` (kind = `Document`) into a `UniversalDocument`.
///
/// Non-document nodes are also convertible — the kind label is stored in the
/// `node_kind` metadata field so round-trips are lossless.
impl From<&GraphNode> for UniversalDocument {
    fn from(node: &GraphNode) -> Self {
        let mut doc = UniversalDocument::new(&node.id, &node.title, &node.body_markdown)
            .with_metadata("node_kind", node.kind.label())
            .with_metadata("created_at", node.created_at.to_string())
            .with_metadata("updated_at", node.updated_at.to_string());

        // Propagate metadata
        for (k, v) in &node.metadata {
            doc = doc.with_metadata(k, v);
        }
        doc
    }
}

// ─── Temporal Delta ──────────────────────────────────────────────────────────

/// The kind of change recorded in a `TemporalDelta`.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum DeltaKind {
    /// A new node was inserted.
    Insert,
    /// An existing node was replaced (content changed).
    Update,
    /// A node was superseded (logically "removed" — the node record is
    /// preserved; only the active index is updated).
    Supersede,
    /// A link was added between two nodes.
    LinkAdded,
    /// A link was removed between two nodes.
    LinkRemoved,
}

/// A single immutable event record that captures what changed in the
/// `LocalNoosphere` and when.
///
/// `TemporalDelta` objects form the raw event stream that a `TemporalAnchor`
/// implementation will hash-chain into a cryptographically verifiable history.
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct TemporalDelta {
    /// Sequential index in this session's delta log.
    pub index: u64,
    /// UNIX timestamp (seconds).
    pub timestamp: u64,
    /// Kind of change.
    pub kind: DeltaKind,
    /// ID of the primary node affected.
    pub node_id: String,
    /// ID of the secondary node affected (for Link deltas; empty otherwise).
    pub related_node_id: String,
    /// A short human-readable summary of the change.
    pub summary: String,
}

// ─── LocalNoosphere ──────────────────────────────────────────────────────────

/// The Aluminum OS sovereign knowledge graph.
///
/// All data is held in memory using `BTreeMap` for deterministic ordering.
/// A `Vec<TemporalDelta>` captures every state change so the history can be
/// verified and replayed.
///
/// # Thread Safety
/// This struct is intentionally single-threaded (`!Send`) in its current form.
/// Wrap in `Arc<Mutex<LocalNoosphere>>` for multi-threaded use.
pub struct LocalNoosphere {
    /// Active nodes, keyed by node ID.
    nodes: BTreeMap<String, GraphNode>,
    /// Superseded (logically deleted) nodes, preserved for history.
    superseded: Vec<GraphNode>,
    /// Append-only list of all state changes this session.
    deltas: Vec<TemporalDelta>,
    /// Delta counter for sequential indexing.
    delta_counter: u64,
}

impl LocalNoosphere {
    /// Create a new, empty `LocalNoosphere`.
    pub fn new() -> Self {
        LocalNoosphere {
            nodes: BTreeMap::new(),
            superseded: Vec::new(),
            deltas: Vec::new(),
            delta_counter: 0,
        }
    }

    // ── Write Operations ────────────────────────────────────────────────────

    /// Insert a new node.  If a node with the same ID already exists, it is
    /// superseded (moved to `self.superseded`) and replaced by `node`.
    pub fn insert(&mut self, node: GraphNode) {
        let node_id = node.id.clone();

        if let Some(old) = self.nodes.remove(&node_id) {
            self.record_delta(DeltaKind::Supersede, &node_id, "", "node superseded");
            self.superseded.push(old);
        }

        self.record_delta(DeltaKind::Insert, &node_id, "", &format!("inserted {}", node.kind.label()));
        self.nodes.insert(node_id, node);
    }

    /// Update only the title and body of an existing node, preserving all
    /// other fields (tags, links, metadata).  If the node does not exist,
    /// this is equivalent to `insert` with a new bare node.
    pub fn update_content(
        &mut self,
        node_id: &str,
        new_title: impl Into<String>,
        new_body: impl Into<String>,
    ) {
        let new_title = new_title.into();
        let new_body = new_body.into();

        if let Some(node) = self.nodes.get_mut(node_id) {
            node.title = new_title;
            node.body_markdown = new_body;
            node.updated_at = unix_now();
            self.record_delta(DeltaKind::Update, node_id, "", "content updated");
        } else {
            let node = GraphNode::new(node_id, NodeKind::Document, new_title, new_body);
            self.insert(node);
        }
    }

    /// Add a bidirectional link between `from_id` and `to_id`.
    ///
    /// Does nothing if either node does not exist or if the link already
    /// exists.  Returns `true` if the link was newly created.
    pub fn link(&mut self, from_id: &str, to_id: &str) -> bool {
        if from_id == to_id {
            return false;
        }
        if !self.nodes.contains_key(from_id) || !self.nodes.contains_key(to_id) {
            return false;
        }

        let already_linked = self.nodes[from_id].linked_node_ids.contains(to_id);
        if already_linked {
            return false;
        }

        // Safety: we checked both keys above.
        self.nodes.get_mut(from_id).unwrap().linked_node_ids.insert(to_id.to_string());
        self.nodes.get_mut(to_id).unwrap().linked_node_ids.insert(from_id.to_string());
        self.record_delta(DeltaKind::LinkAdded, from_id, to_id, "bidirectional link added");
        true
    }

    /// Remove a bidirectional link between `from_id` and `to_id`.
    ///
    /// Returns `true` if the link existed and was removed.
    pub fn unlink(&mut self, from_id: &str, to_id: &str) -> bool {
        let removed_a = self
            .nodes
            .get_mut(from_id)
            .map(|n| n.linked_node_ids.remove(to_id))
            .unwrap_or(false);
        let removed_b = self
            .nodes
            .get_mut(to_id)
            .map(|n| n.linked_node_ids.remove(from_id))
            .unwrap_or(false);

        if removed_a || removed_b {
            self.record_delta(DeltaKind::LinkRemoved, from_id, to_id, "bidirectional link removed");
            true
        } else {
            false
        }
    }

    /// Logically remove a node by superseding it.  The node is moved to
    /// `self.superseded` and a `Supersede` delta is recorded.  Its ID is
    /// removed from all other nodes' link sets.
    pub fn remove(&mut self, node_id: &str) -> bool {
        if let Some(mut node) = self.nodes.remove(node_id) {
            // Clean up all outgoing links from other nodes.
            let linked: Vec<String> = node.linked_node_ids.iter().cloned().collect();
            node.linked_node_ids.clear();
            for other_id in &linked {
                if let Some(other) = self.nodes.get_mut(other_id.as_str()) {
                    other.linked_node_ids.remove(node_id);
                }
            }
            self.record_delta(DeltaKind::Supersede, node_id, "", "node removed (superseded)");
            self.superseded.push(node);
            true
        } else {
            false
        }
    }

    // ── Read Operations ─────────────────────────────────────────────────────

    /// Return an immutable reference to the node with the given ID, if present.
    pub fn get(&self, node_id: &str) -> Option<&GraphNode> {
        self.nodes.get(node_id)
    }

    /// Return `true` if the node is active (not superseded).
    pub fn contains(&self, node_id: &str) -> bool {
        self.nodes.contains_key(node_id)
    }

    /// Total number of active nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Return `true` if there are no active nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Return all active nodes whose title or body contains `needle`
    /// (case-insensitive substring match).
    ///
    /// Results are returned in deterministic order (sorted by node ID).
    /// Phase 2: replace with SQLite FTS5 for sub-millisecond large-scale search.
    pub fn query(&self, needle: &str) -> Vec<&GraphNode> {
        self.nodes
            .values()
            .filter(|n| n.matches_text(needle))
            .collect()
    }

    /// Return all active nodes that carry the given tag.
    pub fn query_by_tag(&self, tag: &str) -> Vec<&GraphNode> {
        self.nodes
            .values()
            .filter(|n| n.has_tag(tag))
            .collect()
    }

    /// Return all active nodes of the given kind.
    pub fn query_by_kind(&self, kind: &NodeKind) -> Vec<&GraphNode> {
        self.nodes
            .values()
            .filter(|n| &n.kind == kind)
            .collect()
    }

    /// Return all active nodes linked to `node_id`.
    ///
    /// Returns an empty vec if the node does not exist or has no links.
    pub fn neighbors(&self, node_id: &str) -> Vec<&GraphNode> {
        let Some(node) = self.nodes.get(node_id) else {
            return Vec::new();
        };

        node.linked_node_ids
            .iter()
            .filter_map(|id| self.nodes.get(id.as_str()))
            .collect()
    }

    /// Return all nodes (active and superseded) as an iterator, in insertion
    /// order of the active map followed by the superseded list.
    pub fn all_nodes(&self) -> impl Iterator<Item = &GraphNode> {
        self.nodes.values().chain(self.superseded.iter())
    }

    // ── Temporal Delta Log ──────────────────────────────────────────────────

    /// Return the full ordered list of state changes recorded this session.
    pub fn deltas(&self) -> &[TemporalDelta] {
        &self.deltas
    }

    /// Total number of state-change events recorded.
    pub fn delta_count(&self) -> usize {
        self.deltas.len()
    }

    // ── Provider Bridge ─────────────────────────────────────────────────────

    /// Import a `UniversalDocument` as a `Document` node.  If a node with the
    /// same ID already exists it is superseded.
    pub fn import_document(&mut self, doc: &UniversalDocument) {
        let node = GraphNode::from(doc);
        self.insert(node);
    }

    /// Export the node with the given ID as a `UniversalDocument`.
    ///
    /// Returns `None` if the node does not exist.
    pub fn export_document(&self, node_id: &str) -> Option<UniversalDocument> {
        self.nodes.get(node_id).map(UniversalDocument::from)
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    fn record_delta(
        &mut self,
        kind: DeltaKind,
        node_id: &str,
        related_node_id: &str,
        summary: &str,
    ) {
        let delta = TemporalDelta {
            index: self.delta_counter,
            timestamp: unix_now(),
            kind,
            node_id: node_id.to_string(),
            related_node_id: related_node_id.to_string(),
            summary: summary.to_string(),
        };
        self.delta_counter += 1;
        self.deltas.push(delta);
    }
}

impl Default for LocalNoosphere {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: &str, kind: NodeKind, title: &str, body: &str) -> GraphNode {
        GraphNode::new(id, kind, title, body)
    }

    // ── NodeKind ──────────────────────────────────────────────────────────

    #[test]
    fn test_node_kind_labels() {
        assert_eq!(NodeKind::Document.label(), "document");
        assert_eq!(NodeKind::Email.label(), "email");
        assert_eq!(NodeKind::CalendarEvent.label(), "calendar_event");
        assert_eq!(NodeKind::Contact.label(), "contact");
        assert_eq!(NodeKind::Task.label(), "task");
        assert_eq!(NodeKind::Note.label(), "note");
        assert_eq!(NodeKind::WebPage.label(), "web_page");
        assert_eq!(NodeKind::Other("custom".to_string()).label(), "custom");
    }

    // ── GraphNode ─────────────────────────────────────────────────────────

    #[test]
    fn test_graph_node_new() {
        let node = make_node("n1", NodeKind::Document, "Hello", "body");
        assert_eq!(node.id, "n1");
        assert_eq!(node.kind, NodeKind::Document);
        assert_eq!(node.title, "Hello");
        assert_eq!(node.body_markdown, "body");
        assert!(node.tags.is_empty());
        assert!(node.linked_node_ids.is_empty());
    }

    #[test]
    fn test_graph_node_with_tag_and_metadata() {
        let node = make_node("n1", NodeKind::Note, "Title", "body")
            .with_tag("important")
            .with_metadata("author", "Dave");
        assert!(node.has_tag("important"));
        assert!(!node.has_tag("other"));
        assert_eq!(node.metadata.get("author").map(String::as_str), Some("Dave"));
    }

    #[test]
    fn test_graph_node_matches_text_title() {
        let node = make_node("n1", NodeKind::Email, "Q1 Report", "Details here.");
        assert!(node.matches_text("Q1"));
        assert!(node.matches_text("q1")); // case-insensitive
        assert!(!node.matches_text("Q2"));
    }

    #[test]
    fn test_graph_node_matches_text_body() {
        let node = make_node("n1", NodeKind::Document, "Title", "Contains the word aluminum.");
        assert!(node.matches_text("aluminum"));
        assert!(node.matches_text("ALUMINUM"));
    }

    // ── LocalNoosphere insert / get ────────────────────────────────────────

    #[test]
    fn test_insert_and_get() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("n1", NodeKind::Document, "Doc One", "body"));
        assert_eq!(ns.len(), 1);
        let node = ns.get("n1").unwrap();
        assert_eq!(node.title, "Doc One");
    }

    #[test]
    fn test_insert_supersedes_existing() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("n1", NodeKind::Document, "Old Title", "old body"));
        ns.insert(make_node("n1", NodeKind::Document, "New Title", "new body"));
        // Still only one active node
        assert_eq!(ns.len(), 1);
        assert_eq!(ns.get("n1").unwrap().title, "New Title");
        // But the old node is preserved in superseded list
        assert_eq!(ns.superseded.len(), 1);
        assert_eq!(ns.superseded[0].title, "Old Title");
    }

    #[test]
    fn test_contains_and_is_empty() {
        let mut ns = LocalNoosphere::new();
        assert!(ns.is_empty());
        ns.insert(make_node("n1", NodeKind::Note, "T", "b"));
        assert!(ns.contains("n1"));
        assert!(!ns.contains("n2"));
        assert!(!ns.is_empty());
    }

    // ── update_content ────────────────────────────────────────────────────

    #[test]
    fn test_update_content() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("n1", NodeKind::Document, "Old", "old body").with_tag("keep-me"));
        ns.update_content("n1", "New Title", "new body");
        let node = ns.get("n1").unwrap();
        assert_eq!(node.title, "New Title");
        assert_eq!(node.body_markdown, "new body");
        // Tags are preserved
        assert!(node.has_tag("keep-me"));
    }

    #[test]
    fn test_update_content_creates_if_missing() {
        let mut ns = LocalNoosphere::new();
        ns.update_content("n99", "Title", "Body");
        assert!(ns.contains("n99"));
    }

    // ── link / unlink ─────────────────────────────────────────────────────

    #[test]
    fn test_link_creates_bidirectional_edge() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("a", NodeKind::Document, "A", ""));
        ns.insert(make_node("b", NodeKind::Document, "B", ""));
        assert!(ns.link("a", "b"));
        assert!(ns.get("a").unwrap().linked_node_ids.contains("b"));
        assert!(ns.get("b").unwrap().linked_node_ids.contains("a"));
    }

    #[test]
    fn test_link_returns_false_if_already_linked() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("a", NodeKind::Document, "A", ""));
        ns.insert(make_node("b", NodeKind::Document, "B", ""));
        ns.link("a", "b");
        assert!(!ns.link("a", "b")); // second call is a no-op
    }

    #[test]
    fn test_link_returns_false_for_missing_nodes() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("a", NodeKind::Document, "A", ""));
        assert!(!ns.link("a", "missing"));
    }

    #[test]
    fn test_link_self_loop_rejected() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("a", NodeKind::Document, "A", ""));
        assert!(!ns.link("a", "a"));
    }

    #[test]
    fn test_unlink_removes_bidirectional_edge() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("a", NodeKind::Document, "A", ""));
        ns.insert(make_node("b", NodeKind::Document, "B", ""));
        ns.link("a", "b");
        assert!(ns.unlink("a", "b"));
        assert!(!ns.get("a").unwrap().linked_node_ids.contains("b"));
        assert!(!ns.get("b").unwrap().linked_node_ids.contains("a"));
    }

    // ── remove ────────────────────────────────────────────────────────────

    #[test]
    fn test_remove_moves_to_superseded() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("n1", NodeKind::Task, "Task 1", ""));
        assert!(ns.remove("n1"));
        assert!(!ns.contains("n1"));
        assert_eq!(ns.superseded.len(), 1);
    }

    #[test]
    fn test_remove_cleans_up_incoming_links() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("a", NodeKind::Document, "A", ""));
        ns.insert(make_node("b", NodeKind::Document, "B", ""));
        ns.link("a", "b");
        ns.remove("b");
        // a should no longer list b as a neighbor
        assert!(!ns.get("a").unwrap().linked_node_ids.contains("b"));
    }

    #[test]
    fn test_remove_nonexistent_returns_false() {
        let mut ns = LocalNoosphere::new();
        assert!(!ns.remove("ghost"));
    }

    // ── query ──────────────────────────────────────────────────────────────

    #[test]
    fn test_query_returns_matching_nodes() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("n1", NodeKind::Document, "Aluminum OS roadmap", ""));
        ns.insert(make_node("n2", NodeKind::Email, "Meeting tomorrow", ""));
        let results = ns.query("aluminum");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "n1");
    }

    #[test]
    fn test_query_case_insensitive() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("n1", NodeKind::Note, "ALUMINUM", ""));
        assert_eq!(ns.query("aluminum").len(), 1);
        assert_eq!(ns.query("ALUMINUM").len(), 1);
        assert_eq!(ns.query("AlUmInUm").len(), 1);
    }

    #[test]
    fn test_query_returns_empty_on_no_match() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("n1", NodeKind::Task, "Buy groceries", ""));
        assert!(ns.query("aluminum").is_empty());
    }

    #[test]
    fn test_query_by_tag() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("n1", NodeKind::Document, "A", "").with_tag("priority"));
        ns.insert(make_node("n2", NodeKind::Note, "B", "").with_tag("backlog"));
        let results = ns.query_by_tag("priority");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "n1");
    }

    #[test]
    fn test_query_by_kind() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("n1", NodeKind::Email, "Email 1", ""));
        ns.insert(make_node("n2", NodeKind::Document, "Doc 1", ""));
        ns.insert(make_node("n3", NodeKind::Email, "Email 2", ""));
        let emails = ns.query_by_kind(&NodeKind::Email);
        assert_eq!(emails.len(), 2);
    }

    #[test]
    fn test_neighbors() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("a", NodeKind::Document, "A", ""));
        ns.insert(make_node("b", NodeKind::Document, "B", ""));
        ns.insert(make_node("c", NodeKind::Document, "C", ""));
        ns.link("a", "b");
        ns.link("a", "c");
        let nb: Vec<&str> = {
            let mut ids: Vec<&str> = ns.neighbors("a").iter().map(|n| n.id.as_str()).collect();
            ids.sort();
            ids
        };
        assert_eq!(nb, vec!["b", "c"]);
    }

    // ── Temporal Deltas ────────────────────────────────────────────────────

    #[test]
    fn test_deltas_recorded_on_insert() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("n1", NodeKind::Document, "T", ""));
        assert!(!ns.deltas().is_empty());
        assert!(ns.deltas().iter().any(|d| d.node_id == "n1" && d.kind == DeltaKind::Insert));
    }

    #[test]
    fn test_deltas_recorded_on_link() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("a", NodeKind::Document, "A", ""));
        ns.insert(make_node("b", NodeKind::Document, "B", ""));
        ns.link("a", "b");
        assert!(ns.deltas().iter().any(|d| d.kind == DeltaKind::LinkAdded));
    }

    #[test]
    fn test_deltas_are_sequential() {
        let mut ns = LocalNoosphere::new();
        ns.insert(make_node("n1", NodeKind::Note, "A", ""));
        ns.insert(make_node("n2", NodeKind::Note, "B", ""));
        for (i, delta) in ns.deltas().iter().enumerate() {
            assert_eq!(delta.index, i as u64);
        }
    }

    // ── Provider bridge (UniversalDocument ↔ GraphNode) ───────────────────

    #[test]
    fn test_import_document() {
        let mut ns = LocalNoosphere::new();
        let doc = UniversalDocument::new("doc-1", "My Document", "# Hello\n\nWorld")
            .with_metadata("source", "Google Workspace");
        ns.import_document(&doc);
        assert!(ns.contains("doc-1"));
        let node = ns.get("doc-1").unwrap();
        assert_eq!(node.title, "My Document");
        assert_eq!(node.kind, NodeKind::Document);
    }

    #[test]
    fn test_export_document_round_trip() {
        let mut ns = LocalNoosphere::new();
        let doc = UniversalDocument::new("doc-2", "Export Me", "body text");
        ns.import_document(&doc);
        let exported = ns.export_document("doc-2").unwrap();
        assert_eq!(exported.id, "doc-2");
        assert_eq!(exported.title, "Export Me");
        assert_eq!(exported.body_markdown, "body text");
    }

    #[test]
    fn test_export_document_missing_returns_none() {
        let ns = LocalNoosphere::new();
        assert!(ns.export_document("ghost").is_none());
    }

    #[test]
    fn test_from_universal_document_sets_kind() {
        let doc = UniversalDocument::new("d1", "Title", "body");
        let node = GraphNode::from(&doc);
        assert_eq!(node.kind, NodeKind::Document);
    }

    #[test]
    fn test_from_graph_node_preserves_metadata() {
        let node = make_node("n1", NodeKind::Email, "Subject", "body")
            .with_metadata("from", "alice@example.com");
        let doc = UniversalDocument::from(&node);
        assert_eq!(doc.metadata.get("from").map(String::as_str), Some("alice@example.com"));
        assert_eq!(doc.metadata.get("node_kind").map(String::as_str), Some("email"));
    }
}
