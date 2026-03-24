// src/data_lineage.rs
// Aluminum OS — Data Provenance & Lineage Tracker
//
// Novel Invention #14 — Data Lineage Tracker
//
// When data moves between providers — a Google Doc is copied to Notion,
// a GitHub issue is synced to Linear, an email thread becomes a task —
// the lineage of that data is lost. The user has no way to know where
// something came from, what it was derived from, or what it influenced.
//
// The Data Lineage Tracker solves this by building a directed acyclic
// graph (DAG) of provenance relationships. Every data operation creates
// an edge in this graph: "this Notion page was derived from that Google Doc."
//
// This enables:
// - Auditing: who changed what, based on what source
// - Impact analysis: if I delete this, what else breaks?
// - Rollback: trace back to the original source
// - Compliance: full chain of custody for regulated data
//
// Author: GitHub Copilot — Aluminum OS Session 2026-03-23

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

// ─── Asset identifier ─────────────────────────────────────────────────────

/// A globally unique reference to a data asset across any provider.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetRef {
    /// Provider name (e.g., "github", "notion", "google-drive").
    pub provider: String,
    /// Provider-native ID (e.g., file ID, page ID, issue ID).
    pub id: String,
    /// Human-readable type (e.g., "issue", "page", "file", "email").
    pub kind: String,
}

impl AssetRef {
    pub fn new(provider: impl Into<String>, id: impl Into<String>, kind: impl Into<String>) -> Self {
        AssetRef {
            provider: provider.into(),
            id: id.into(),
            kind: kind.into(),
        }
    }

    pub fn canonical_id(&self) -> String {
        format!("{}:{}:{}", self.provider, self.kind, self.id)
    }
}

// ─── Provenance edge ──────────────────────────────────────────────────────

/// The type of relationship between two assets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeKind {
    /// `target` was copied from `source`.
    CopiedFrom,
    /// `target` was derived/summarized from `source`.
    DerivedFrom,
    /// `target` was synced from `source`.
    SyncedFrom,
    /// `target` references `source`.
    References,
    /// `target` was created by transforming `source`.
    TransformedFrom,
    /// Custom relationship.
    Custom(String),
}

impl EdgeKind {
    pub fn as_str(&self) -> &str {
        match self {
            EdgeKind::CopiedFrom => "copied_from",
            EdgeKind::DerivedFrom => "derived_from",
            EdgeKind::SyncedFrom => "synced_from",
            EdgeKind::References => "references",
            EdgeKind::TransformedFrom => "transformed_from",
            EdgeKind::Custom(s) => s,
        }
    }
}

/// A directed provenance edge: `source` → `target` with a relationship type.
#[derive(Debug, Clone)]
pub struct ProvenanceEdge {
    pub source: AssetRef,
    pub target: AssetRef,
    pub kind: EdgeKind,
    /// ISO 8601 timestamp when this relationship was established.
    pub timestamp: String,
    /// Actor who caused this relationship (username/email).
    pub actor: String,
    /// Optional annotation.
    pub notes: Option<String>,
}

// ─── Lineage graph ────────────────────────────────────────────────────────

/// The in-memory provenance graph.
#[derive(Debug, Default, Clone)]
pub struct LineageGraph {
    /// All edges in the graph.
    edges: Vec<ProvenanceEdge>,
    /// Index: canonical_id → outgoing edge indices.
    outgoing: BTreeMap<String, Vec<usize>>,
    /// Index: canonical_id → incoming edge indices.
    incoming: BTreeMap<String, Vec<usize>>,
}

impl LineageGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a new provenance relationship.
    pub fn record(&mut self, edge: ProvenanceEdge) {
        let idx = self.edges.len();
        let src_key = edge.source.canonical_id();
        let tgt_key = edge.target.canonical_id();
        self.outgoing.entry(src_key).or_default().push(idx);
        self.incoming.entry(tgt_key).or_default().push(idx);
        self.edges.push(edge);
    }

    /// Get all edges where `asset` is the source (what did this asset produce?).
    pub fn descendants_of(&self, asset: &AssetRef) -> Vec<&ProvenanceEdge> {
        let key = asset.canonical_id();
        self.outgoing
            .get(&key)
            .map(|idxs| idxs.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    /// Get all edges where `asset` is the target (where did this asset come from?).
    pub fn ancestors_of(&self, asset: &AssetRef) -> Vec<&ProvenanceEdge> {
        let key = asset.canonical_id();
        self.incoming
            .get(&key)
            .map(|idxs| idxs.iter().map(|&i| &self.edges[i]).collect())
            .unwrap_or_default()
    }

    /// Trace the full lineage chain back to the ultimate origin of an asset.
    pub fn trace_to_origin(&self, asset: &AssetRef) -> Vec<AssetRef> {
        let mut path = Vec::new();
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(asset.clone());

        while let Some(current) = queue.pop_front() {
            let key = current.canonical_id();
            if visited.contains(&key) {
                continue;
            }
            visited.insert(key);
            path.push(current.clone());

            for edge in self.ancestors_of(&current) {
                queue.push_back(edge.source.clone());
            }
        }
        path
    }

    /// Compute the impact: all assets that would be affected if `asset` is changed.
    pub fn impact_analysis(&self, asset: &AssetRef) -> Vec<AssetRef> {
        let mut affected = Vec::new();
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(asset.clone());

        while let Some(current) = queue.pop_front() {
            let key = current.canonical_id();
            if visited.contains(&key) {
                continue;
            }
            visited.insert(key.clone());

            for edge in self.descendants_of(&current) {
                if !visited.contains(&edge.target.canonical_id()) {
                    affected.push(edge.target.clone());
                    queue.push_back(edge.target.clone());
                }
            }
        }
        affected
    }

    /// Get all assets in the graph.
    pub fn all_assets(&self) -> BTreeSet<String> {
        let mut assets = BTreeSet::new();
        for edge in &self.edges {
            assets.insert(edge.source.canonical_id());
            assets.insert(edge.target.canonical_id());
        }
        assets
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn asset_count(&self) -> usize {
        self.all_assets().len()
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn drive_file(id: &str) -> AssetRef {
        AssetRef::new("google-drive", id, "file")
    }

    fn notion_page(id: &str) -> AssetRef {
        AssetRef::new("notion", id, "page")
    }

    fn github_issue(id: &str) -> AssetRef {
        AssetRef::new("github", id, "issue")
    }

    fn make_edge(src: AssetRef, tgt: AssetRef, kind: EdgeKind) -> ProvenanceEdge {
        ProvenanceEdge {
            source: src,
            target: tgt,
            kind,
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            actor: "alice".to_string(),
            notes: None,
        }
    }

    #[test]
    fn test_asset_ref_canonical_id() {
        let a = drive_file("file123");
        assert_eq!(a.canonical_id(), "google-drive:file:file123");
    }

    #[test]
    fn test_edge_kind_as_str() {
        assert_eq!(EdgeKind::CopiedFrom.as_str(), "copied_from");
        assert_eq!(EdgeKind::DerivedFrom.as_str(), "derived_from");
        assert_eq!(EdgeKind::Custom("my_edge".to_string()).as_str(), "my_edge");
    }

    #[test]
    fn test_graph_record_and_count() {
        let mut g = LineageGraph::new();
        g.record(make_edge(drive_file("f1"), notion_page("p1"), EdgeKind::CopiedFrom));
        assert_eq!(g.edge_count(), 1);
        assert_eq!(g.asset_count(), 2);
    }

    #[test]
    fn test_descendants_of() {
        let mut g = LineageGraph::new();
        let src = drive_file("f1");
        let tgt1 = notion_page("p1");
        let tgt2 = notion_page("p2");
        g.record(make_edge(src.clone(), tgt1, EdgeKind::CopiedFrom));
        g.record(make_edge(src.clone(), tgt2, EdgeKind::CopiedFrom));
        let descendants = g.descendants_of(&src);
        assert_eq!(descendants.len(), 2);
    }

    #[test]
    fn test_ancestors_of() {
        let mut g = LineageGraph::new();
        let src = drive_file("f1");
        let tgt = notion_page("p1");
        g.record(make_edge(src.clone(), tgt.clone(), EdgeKind::CopiedFrom));
        let ancestors = g.ancestors_of(&tgt);
        assert_eq!(ancestors.len(), 1);
        assert_eq!(ancestors[0].source.canonical_id(), src.canonical_id());
    }

    #[test]
    fn test_trace_to_origin_chain() {
        let mut g = LineageGraph::new();
        let a = drive_file("original");
        let b = notion_page("copy");
        let c = github_issue("summary");
        g.record(make_edge(a.clone(), b.clone(), EdgeKind::CopiedFrom));
        g.record(make_edge(b.clone(), c.clone(), EdgeKind::DerivedFrom));

        // Tracing c back should reach a
        let chain = g.trace_to_origin(&c);
        let ids: Vec<String> = chain.iter().map(|a| a.canonical_id()).collect();
        assert!(ids.contains(&a.canonical_id()));
    }

    #[test]
    fn test_impact_analysis() {
        let mut g = LineageGraph::new();
        let root = drive_file("root");
        let child1 = notion_page("child1");
        let child2 = notion_page("child2");
        let grandchild = github_issue("grandchild");
        g.record(make_edge(root.clone(), child1.clone(), EdgeKind::SyncedFrom));
        g.record(make_edge(root.clone(), child2.clone(), EdgeKind::SyncedFrom));
        g.record(make_edge(child1.clone(), grandchild.clone(), EdgeKind::DerivedFrom));

        let impact = g.impact_analysis(&root);
        assert_eq!(impact.len(), 3); // child1, child2, grandchild
    }

    #[test]
    fn test_no_ancestors_for_root() {
        let mut g = LineageGraph::new();
        let root = drive_file("root");
        let child = notion_page("child");
        g.record(make_edge(root.clone(), child, EdgeKind::CopiedFrom));
        let ancestors = g.ancestors_of(&root);
        assert!(ancestors.is_empty());
    }

    #[test]
    fn test_trace_origin_already_root() {
        let g = LineageGraph::new();
        let root = drive_file("root");
        let chain = g.trace_to_origin(&root);
        // Even with no edges, the asset itself is in the chain
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].canonical_id(), root.canonical_id());
    }

    #[test]
    fn test_all_assets() {
        let mut g = LineageGraph::new();
        g.record(make_edge(drive_file("f1"), notion_page("p1"), EdgeKind::CopiedFrom));
        g.record(make_edge(notion_page("p1"), github_issue("i1"), EdgeKind::DerivedFrom));
        let assets = g.all_assets();
        assert_eq!(assets.len(), 3);
    }
}
