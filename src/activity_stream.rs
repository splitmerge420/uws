// src/activity_stream.rs
// Aluminum OS — Unified Cross-Provider Activity Stream
//
// Novel Invention #11 — Activity Stream
//
// Every provider generates events: new GitHub commits, new Slack messages,
// new Linear issues, new Notion pages. This module normalizes them all into
// a single, chronologically ordered feed — the universal activity stream.
//
// The activity stream is the OS-level "what happened across my digital life"
// query, analogous to a bank statement but for all productivity events.
//
// Commands:
//   uws activity list --params '{"limit":"20"}'
//   uws activity list --params '{"provider":"github,slack","since":"2026-01-01T00:00:00Z"}'
//   uws activity search --params '{"q":"bug fix"}'
//
// Author: GitHub Copilot — Aluminum OS Session 2026-03-23

#![allow(dead_code)]

use std::collections::BTreeMap;

// ─── Event types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventKind {
    // GitHub
    GitHubCommit,
    GitHubIssueOpened,
    GitHubIssueClosed,
    GitHubPrOpened,
    GitHubPrMerged,
    GitHubRelease,
    GitHubReview,
    // Slack
    SlackMessage,
    SlackReaction,
    SlackChannelJoin,
    // Linear
    LinearIssueCreated,
    LinearIssueUpdated,
    LinearCycleStarted,
    // Notion
    NotionPageCreated,
    NotionPageUpdated,
    NotionCommentAdded,
    // Email
    GmailReceived,
    GmailSent,
    OutlookReceived,
    OutlookSent,
    // Calendar
    CalendarEventCreated,
    CalendarEventUpdated,
    // Files
    DriveFileUploaded,
    DriveFileShared,
    OneDriveFileUploaded,
    // Generic
    Unknown(String),
}

impl EventKind {
    pub fn as_str(&self) -> &str {
        match self {
            EventKind::GitHubCommit => "github.commit",
            EventKind::GitHubIssueOpened => "github.issue.opened",
            EventKind::GitHubIssueClosed => "github.issue.closed",
            EventKind::GitHubPrOpened => "github.pr.opened",
            EventKind::GitHubPrMerged => "github.pr.merged",
            EventKind::GitHubRelease => "github.release",
            EventKind::GitHubReview => "github.review",
            EventKind::SlackMessage => "slack.message",
            EventKind::SlackReaction => "slack.reaction",
            EventKind::SlackChannelJoin => "slack.channel.join",
            EventKind::LinearIssueCreated => "linear.issue.created",
            EventKind::LinearIssueUpdated => "linear.issue.updated",
            EventKind::LinearCycleStarted => "linear.cycle.started",
            EventKind::NotionPageCreated => "notion.page.created",
            EventKind::NotionPageUpdated => "notion.page.updated",
            EventKind::NotionCommentAdded => "notion.comment.added",
            EventKind::GmailReceived => "gmail.received",
            EventKind::GmailSent => "gmail.sent",
            EventKind::OutlookReceived => "outlook.received",
            EventKind::OutlookSent => "outlook.sent",
            EventKind::CalendarEventCreated => "calendar.event.created",
            EventKind::CalendarEventUpdated => "calendar.event.updated",
            EventKind::DriveFileUploaded => "drive.file.uploaded",
            EventKind::DriveFileShared => "drive.file.shared",
            EventKind::OneDriveFileUploaded => "onedrive.file.uploaded",
            EventKind::Unknown(s) => s,
        }
    }

    /// Parse an event kind string back to enum.
    pub fn parse(s: &str) -> EventKind {
        match s {
            "github.commit" => EventKind::GitHubCommit,
            "github.issue.opened" => EventKind::GitHubIssueOpened,
            "github.issue.closed" => EventKind::GitHubIssueClosed,
            "github.pr.opened" => EventKind::GitHubPrOpened,
            "github.pr.merged" => EventKind::GitHubPrMerged,
            "github.release" => EventKind::GitHubRelease,
            "slack.message" => EventKind::SlackMessage,
            "linear.issue.created" => EventKind::LinearIssueCreated,
            "notion.page.created" => EventKind::NotionPageCreated,
            "gmail.received" => EventKind::GmailReceived,
            other => EventKind::Unknown(other.to_string()),
        }
    }

    /// The provider that generates this event kind.
    pub fn provider(&self) -> &str {
        let s = self.as_str();
        s.split('.').next().unwrap_or("unknown")
    }
}

// ─── Activity event ───────────────────────────────────────────────────────

/// A single normalized activity event from any provider.
#[derive(Debug, Clone)]
pub struct ActivityEvent {
    /// Globally unique event ID (provider:kind:id).
    pub id: String,
    pub kind: EventKind,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Actor who caused the event (username or email).
    pub actor: String,
    /// Short human-readable summary.
    pub summary: String,
    /// Link to the event (URL).
    pub url: String,
    /// Provider-specific extra fields.
    pub metadata: BTreeMap<String, String>,
}

impl ActivityEvent {
    pub fn new(
        kind: EventKind,
        timestamp: impl Into<String>,
        actor: impl Into<String>,
        summary: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        let kind_str = kind.as_str().to_string();
        let actor_s: String = actor.into();
        let ts: String = timestamp.into();
        let id = format!("{}:{}:{}", kind.provider(), kind_str, ts.replace(':', "-"));
        ActivityEvent {
            id,
            kind,
            timestamp: ts,
            actor: actor_s,
            summary: summary.into(),
            url: url.into(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

// ─── Activity stream ──────────────────────────────────────────────────────

/// An in-memory ring-buffer of activity events.
#[derive(Debug, Default, Clone)]
pub struct ActivityStream {
    events: Vec<ActivityEvent>,
    /// Max events to retain (older events are dropped).
    capacity: usize,
}

impl ActivityStream {
    pub fn new(capacity: usize) -> Self {
        ActivityStream {
            events: Vec::new(),
            capacity,
        }
    }

    /// Ingest an event into the stream.
    pub fn push(&mut self, event: ActivityEvent) {
        self.events.push(event);
        if self.events.len() > self.capacity {
            self.events.remove(0);
        }
    }

    /// Ingest multiple events (e.g. from a provider batch response).
    pub fn push_many(&mut self, events: Vec<ActivityEvent>) {
        for e in events {
            self.push(e);
        }
    }

    /// Return events sorted by timestamp descending (most recent first).
    pub fn sorted(&self) -> Vec<&ActivityEvent> {
        let mut sorted: Vec<&ActivityEvent> = self.events.iter().collect();
        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sorted
    }

    /// Return the N most recent events.
    pub fn latest(&self, n: usize) -> Vec<&ActivityEvent> {
        self.sorted().into_iter().take(n).collect()
    }

    /// Filter by provider name (e.g. "github", "slack").
    pub fn by_provider(&self, provider: &str) -> Vec<&ActivityEvent> {
        self.events
            .iter()
            .filter(|e| e.kind.provider() == provider)
            .collect()
    }

    /// Filter by event kind.
    pub fn by_kind(&self, kind: &EventKind) -> Vec<&ActivityEvent> {
        self.events.iter().filter(|e| &e.kind == kind).collect()
    }

    /// Filter events after a given ISO 8601 timestamp.
    pub fn since(&self, timestamp: &str) -> Vec<&ActivityEvent> {
        self.events
            .iter()
            .filter(|e| e.timestamp.as_str() >= timestamp)
            .collect()
    }

    /// Full-text search across summary and metadata values.
    pub fn search(&self, query: &str) -> Vec<&ActivityEvent> {
        let q = query.to_lowercase();
        self.events
            .iter()
            .filter(|e| {
                e.summary.to_lowercase().contains(&q)
                    || e.actor.to_lowercase().contains(&q)
                    || e.metadata.values().any(|v| v.to_lowercase().contains(&q))
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(kind: EventKind, ts: &str, actor: &str, summary: &str) -> ActivityEvent {
        ActivityEvent::new(kind, ts, actor, summary, "https://example.com")
    }

    #[test]
    fn test_event_kind_as_str() {
        assert_eq!(EventKind::GitHubCommit.as_str(), "github.commit");
        assert_eq!(EventKind::SlackMessage.as_str(), "slack.message");
        assert_eq!(EventKind::LinearIssueCreated.as_str(), "linear.issue.created");
    }

    #[test]
    fn test_event_kind_provider() {
        assert_eq!(EventKind::GitHubCommit.provider(), "github");
        assert_eq!(EventKind::SlackMessage.provider(), "slack");
        assert_eq!(EventKind::NotionPageCreated.provider(), "notion");
    }

    #[test]
    fn test_event_kind_from_str_roundtrip() {
        let original = EventKind::GitHubPrMerged;
        let parsed = EventKind::parse(original.as_str());
        assert_eq!(parsed.as_str(), "github.pr.merged");
    }

    #[test]
    fn test_event_kind_from_str_unknown() {
        let k = EventKind::parse("unknown.event.xyz");
        if let EventKind::Unknown(s) = k {
            assert_eq!(s, "unknown.event.xyz");
        } else {
            panic!("Expected Unknown variant");
        }
    }

    #[test]
    fn test_activity_event_new() {
        let e = make_event(EventKind::GitHubCommit, "2026-01-01T10:00:00Z", "alice", "Fix bug");
        assert_eq!(e.kind, EventKind::GitHubCommit);
        assert_eq!(e.actor, "alice");
        assert!(!e.id.is_empty());
    }

    #[test]
    fn test_activity_event_with_metadata() {
        let e = make_event(EventKind::GitHubCommit, "2026-01-01T00:00:00Z", "alice", "test")
            .with_metadata("repo", "octocat/Hello-World")
            .with_metadata("sha", "abc1234");
        assert_eq!(e.metadata.get("repo").map(|s| s.as_str()), Some("octocat/Hello-World"));
    }

    #[test]
    fn test_stream_push_and_len() {
        let mut stream = ActivityStream::new(100);
        stream.push(make_event(EventKind::GitHubCommit, "2026-01-01T00:00:00Z", "alice", "Fix"));
        assert_eq!(stream.len(), 1);
    }

    #[test]
    fn test_stream_capacity_enforced() {
        let mut stream = ActivityStream::new(3);
        for i in 0..5 {
            stream.push(make_event(
                EventKind::SlackMessage,
                &format!("2026-01-{:02}T00:00:00Z", i + 1),
                "alice",
                &format!("msg {i}"),
            ));
        }
        assert_eq!(stream.len(), 3);
    }

    #[test]
    fn test_stream_sorted_most_recent_first() {
        let mut stream = ActivityStream::new(100);
        stream.push(make_event(EventKind::GitHubCommit, "2026-01-01T00:00:00Z", "a", "old"));
        stream.push(make_event(EventKind::SlackMessage, "2026-03-01T00:00:00Z", "b", "new"));
        let sorted = stream.sorted();
        assert!(sorted[0].timestamp > sorted[1].timestamp);
    }

    #[test]
    fn test_stream_latest() {
        let mut stream = ActivityStream::new(100);
        for i in 0..10 {
            stream.push(make_event(
                EventKind::GitHubCommit,
                &format!("2026-01-{:02}T00:00:00Z", i + 1),
                "alice",
                &format!("commit {i}"),
            ));
        }
        assert_eq!(stream.latest(3).len(), 3);
    }

    #[test]
    fn test_stream_by_provider() {
        let mut stream = ActivityStream::new(100);
        stream.push(make_event(EventKind::GitHubCommit, "2026-01-01T00:00:00Z", "a", "commit"));
        stream.push(make_event(EventKind::SlackMessage, "2026-01-02T00:00:00Z", "b", "msg"));
        let github_events = stream.by_provider("github");
        assert_eq!(github_events.len(), 1);
    }

    #[test]
    fn test_stream_by_kind() {
        let mut stream = ActivityStream::new(100);
        stream.push(make_event(EventKind::GitHubCommit, "2026-01-01T00:00:00Z", "a", "commit"));
        stream.push(make_event(EventKind::GitHubPrMerged, "2026-01-02T00:00:00Z", "b", "pr"));
        let commits = stream.by_kind(&EventKind::GitHubCommit);
        assert_eq!(commits.len(), 1);
    }

    #[test]
    fn test_stream_since_filter() {
        let mut stream = ActivityStream::new(100);
        stream.push(make_event(EventKind::GmailReceived, "2026-01-01T00:00:00Z", "a", "old email"));
        stream.push(make_event(EventKind::GmailReceived, "2026-03-01T00:00:00Z", "b", "new email"));
        let recent = stream.since("2026-02-01T00:00:00Z");
        assert_eq!(recent.len(), 1);
        assert!(recent[0].summary.contains("new"));
    }

    #[test]
    fn test_stream_search() {
        let mut stream = ActivityStream::new(100);
        stream.push(make_event(EventKind::SlackMessage, "2026-01-01T00:00:00Z", "alice", "budget review Q4"));
        stream.push(make_event(EventKind::GitHubCommit, "2026-01-02T00:00:00Z", "bob", "fix login bug"));
        let results = stream.search("budget");
        assert_eq!(results.len(), 1);
        assert!(results[0].summary.contains("budget"));
    }

    #[test]
    fn test_push_many() {
        let mut stream = ActivityStream::new(100);
        let events = vec![
            make_event(EventKind::GitHubCommit, "2026-01-01T00:00:00Z", "a", "c1"),
            make_event(EventKind::GitHubCommit, "2026-01-02T00:00:00Z", "a", "c2"),
            make_event(EventKind::GitHubCommit, "2026-01-03T00:00:00Z", "a", "c3"),
        ];
        stream.push_many(events);
        assert_eq!(stream.len(), 3);
    }
}
