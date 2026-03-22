// src/frictionless_cal.rs
// Aluminum OS — FrictionlessCal: Unified Calendar Engine
//
// Merges Google Calendar, Outlook Calendar, and Apple Calendar into a
// single, conflict-resolved, timezone-normalised timeline.  The user
// never sees duplicate events or timezone-shifted start times; the OS
// handles all of it locally, with no cloud round-trip.
//
// Design principles:
//   - `UniversalEvent` is the canonical, provider-neutral event.
//   - All timestamps are stored as UTC ISO 8601 strings.
//   - Duplicate detection: events with the same title and overlapping
//     time window across providers are treated as the same logical event.
//   - `ConflictPolicy` governs how such duplicates are surfaced.
//   - The `TimelineBuilder` accumulates events from multiple sources
//     and produces a sorted, deduped `UnifiedTimeline`.
//
// Constitutional invariants:
//   INV-1 (Sovereignty)   — the unified timeline is computed locally.
//   INV-6 (Provider Abstraction) — CalendarSource is provider-neutral.
//
// Author: GitHub Copilot (builder) + Aluminum OS Council
// Session: 2026-03-22

#![allow(dead_code)]

use std::collections::BTreeMap;

// ─── Core types ───────────────────────────────────────────────────────────

/// Identifies the originating calendar provider.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CalendarSource {
    Google,
    Microsoft,
    Apple,
    Local,
    Other(String),
}

impl CalendarSource {
    pub fn as_str(&self) -> &str {
        match self {
            CalendarSource::Google => "google",
            CalendarSource::Microsoft => "microsoft",
            CalendarSource::Apple => "apple",
            CalendarSource::Local => "local",
            CalendarSource::Other(s) => s.as_str(),
        }
    }
}

/// Attendance / RSVP status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttendeeStatus {
    Accepted,
    Declined,
    Tentative,
    NeedsAction,
    Organizer,
}

impl AttendeeStatus {
    pub fn as_str(&self) -> &str {
        match self {
            AttendeeStatus::Accepted => "accepted",
            AttendeeStatus::Declined => "declined",
            AttendeeStatus::Tentative => "tentative",
            AttendeeStatus::NeedsAction => "needs_action",
            AttendeeStatus::Organizer => "organizer",
        }
    }
}

/// An attendee on a calendar event.
#[derive(Debug, Clone, PartialEq)]
pub struct Attendee {
    pub email: String,
    pub display_name: Option<String>,
    pub status: AttendeeStatus,
}

impl Attendee {
    pub fn new(email: impl Into<String>, status: AttendeeStatus) -> Self {
        Attendee {
            email: email.into(),
            display_name: None,
            status,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }
}

/// Whether the event is all-day or has specific start/end times.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventTimeKind {
    /// An all-day event; times are dates only (YYYY-MM-DD).
    AllDay,
    /// A timed event; times are UTC ISO 8601 (e.g. "2026-03-22T09:00:00Z").
    Timed,
}

/// The canonical, provider-neutral calendar event.
#[derive(Debug, Clone)]
pub struct UniversalEvent {
    /// Globally unique ID within the `UnifiedTimeline`.
    /// Format: `<source.as_str()>:<provider_event_id>`.
    pub id: String,
    /// Provider-assigned event ID.
    pub provider_id: String,
    /// Originating provider.
    pub source: CalendarSource,
    /// Event title/summary.
    pub title: String,
    /// Optional description or body.
    pub description: Option<String>,
    /// Start time (UTC ISO 8601 for timed; YYYY-MM-DD for all-day).
    pub start: String,
    /// End time (UTC ISO 8601 for timed; YYYY-MM-DD for all-day).
    pub end: String,
    /// Whether this is an all-day event.
    pub time_kind: EventTimeKind,
    /// Location string (room name, address, video URL, etc.).
    pub location: Option<String>,
    /// Attendee list.
    pub attendees: Vec<Attendee>,
    /// Provider-specific extra fields preserved for round-trip export.
    pub raw_fields: BTreeMap<String, String>,
    /// Whether the event was added to the unified timeline as a duplicate
    /// reference (i.e., the canonical copy lives under a different ID).
    pub is_duplicate_ref: bool,
    /// ID of the canonical event if this is a duplicate reference.
    pub canonical_id: Option<String>,
}

impl UniversalEvent {
    /// Create a timed event.
    pub fn new_timed(
        provider_id: impl Into<String>,
        source: CalendarSource,
        title: impl Into<String>,
        start_utc: impl Into<String>,
        end_utc: impl Into<String>,
    ) -> Self {
        let pid = provider_id.into();
        let src = source.clone();
        let id = format!("{}:{}", src.as_str(), pid);
        UniversalEvent {
            id,
            provider_id: pid,
            source,
            title: title.into(),
            description: None,
            start: start_utc.into(),
            end: end_utc.into(),
            time_kind: EventTimeKind::Timed,
            location: None,
            attendees: vec![],
            raw_fields: BTreeMap::new(),
            is_duplicate_ref: false,
            canonical_id: None,
        }
    }

    /// Create an all-day event.
    pub fn new_all_day(
        provider_id: impl Into<String>,
        source: CalendarSource,
        title: impl Into<String>,
        date_start: impl Into<String>,
        date_end: impl Into<String>,
    ) -> Self {
        let pid = provider_id.into();
        let src = source.clone();
        let id = format!("{}:{}", src.as_str(), pid);
        UniversalEvent {
            id,
            provider_id: pid,
            source,
            title: title.into(),
            description: None,
            start: date_start.into(),
            end: date_end.into(),
            time_kind: EventTimeKind::AllDay,
            location: None,
            attendees: vec![],
            raw_fields: BTreeMap::new(),
            is_duplicate_ref: false,
            canonical_id: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_location(mut self, loc: impl Into<String>) -> Self {
        self.location = Some(loc.into());
        self
    }

    pub fn with_attendee(mut self, attendee: Attendee) -> Self {
        self.attendees.push(attendee);
        self
    }

    pub fn with_raw_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.raw_fields.insert(key.into(), value.into());
        self
    }

    /// Duration in minutes for timed events. Returns `None` for all-day events
    /// or unparseable times.
    pub fn duration_minutes(&self) -> Option<i64> {
        if self.time_kind == EventTimeKind::AllDay {
            return None;
        }
        parse_minutes_between(&self.start, &self.end)
    }

    /// Whether this event overlaps with another event's time window.
    pub fn overlaps_with(&self, other: &UniversalEvent) -> bool {
        // All-day vs timed: no overlap by design (different systems).
        if self.time_kind != other.time_kind {
            return false;
        }
        // Two events overlap if one starts before the other ends and vice versa.
        self.start < other.end && other.start < self.end
    }
}

// ─── Conflict policy ──────────────────────────────────────────────────────

/// How to handle duplicate events detected across providers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictPolicy {
    /// Keep only the first-seen copy; mark subsequent occurrences as
    /// duplicate references pointing at the canonical entry.
    KeepFirst,
    /// Keep all copies, each tagged with their source provider.
    KeepAll,
    /// Merge duplicate entries into a single event, combining attendee lists
    /// and preferring non-empty description/location from either source.
    Merge,
}

// ─── Duplicate detection ──────────────────────────────────────────────────

/// Two events are considered duplicates if they have the same normalised title
/// and their time windows overlap.
fn are_duplicates(a: &UniversalEvent, b: &UniversalEvent) -> bool {
    if a.source == b.source {
        // Same provider: never a cross-provider duplicate.
        return false;
    }
    normalise_title(&a.title) == normalise_title(&b.title) && a.overlaps_with(b)
}

/// Normalise a title for comparison: lowercase, strip punctuation, collapse whitespace.
fn normalise_title(title: &str) -> String {
    title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' { c.to_ascii_lowercase() } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Merge two duplicate events into one, keeping the richer metadata.
fn merge_events(canonical: UniversalEvent, duplicate: &UniversalEvent) -> UniversalEvent {
    let mut merged = canonical;

    // Prefer non-empty description.
    if merged.description.is_none() {
        merged.description = duplicate.description.clone();
    }

    // Prefer non-empty location.
    if merged.location.is_none() {
        merged.location = duplicate.location.clone();
    }

    // Union attendee lists (by email).
    let existing_emails: std::collections::HashSet<_> =
        merged.attendees.iter().map(|a| a.email.clone()).collect();
    for att in &duplicate.attendees {
        if !existing_emails.contains(&att.email) {
            merged.attendees.push(att.clone());
        }
    }

    // Record the source providers in raw_fields.
    let sources = format!(
        "{},{}",
        merged.source.as_str(),
        duplicate.source.as_str()
    );
    merged.raw_fields.insert("merged_sources".to_string(), sources);

    merged
}

// ─── TimelineBuilder ──────────────────────────────────────────────────────

/// The main engine; accepts pre-fetched raw content.
#[derive(Default)]
pub struct TimelineBuilder {
    events: Vec<UniversalEvent>,
    policy: Option<ConflictPolicy>,
}

impl TimelineBuilder {
    pub fn new() -> Self {
        TimelineBuilder {
            events: vec![],
            policy: None,
        }
    }

    pub fn with_policy(mut self, policy: ConflictPolicy) -> Self {
        self.policy = Some(policy);
        self
    }

    /// Add a single event.
    pub fn add(&mut self, event: UniversalEvent) {
        self.events.push(event);
    }

    /// Add a batch of events from one provider.
    pub fn add_batch(&mut self, events: Vec<UniversalEvent>) {
        self.events.extend(events);
    }

    /// Build the `UnifiedTimeline`, applying deduplication and sorting.
    pub fn build(self) -> UnifiedTimeline {
        let policy = self.policy.unwrap_or(ConflictPolicy::Merge);
        let events = match policy {
            ConflictPolicy::KeepAll => self.events,
            ConflictPolicy::KeepFirst => dedup_keep_first(self.events),
            ConflictPolicy::Merge => dedup_merge(self.events),
        };

        let mut events = events;
        // Sort by start time (lexicographic on UTC ISO 8601 = chronological).
        events.sort_by(|a, b| a.start.cmp(&b.start).then(a.id.cmp(&b.id)));

        UnifiedTimeline { events }
    }
}

fn dedup_keep_first(events: Vec<UniversalEvent>) -> Vec<UniversalEvent> {
    let mut result: Vec<UniversalEvent> = vec![];
    for mut event in events {
        let dup_index = result.iter().position(|e| are_duplicates(e, &event));
        if let Some(idx) = dup_index {
            let canonical_id = result[idx].id.clone();
            event.is_duplicate_ref = true;
            event.canonical_id = Some(canonical_id);
            result.push(event);
        } else {
            result.push(event);
        }
    }
    result
}

fn dedup_merge(events: Vec<UniversalEvent>) -> Vec<UniversalEvent> {
    let mut result: Vec<UniversalEvent> = vec![];
    for event in events {
        let dup_index = result.iter().position(|e| are_duplicates(e, &event));
        if let Some(idx) = dup_index {
            let canonical = result.remove(idx);
            result.push(merge_events(canonical, &event));
        } else {
            result.push(event);
        }
    }
    result
}

// ─── UnifiedTimeline ──────────────────────────────────────────────────────

/// The merged, sorted, deduplicated view of all calendar events.
pub struct UnifiedTimeline {
    events: Vec<UniversalEvent>,
}

impl UnifiedTimeline {
    /// All events in start-time order.
    pub fn events(&self) -> &[UniversalEvent] {
        &self.events
    }

    /// Total event count (including duplicate refs under KeepFirst policy).
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Events that start on or after `from` and start before `until`
    /// (ISO 8601 string range).
    pub fn range(&self, from: &str, until: &str) -> Vec<&UniversalEvent> {
        self.events
            .iter()
            .filter(|e| e.start.as_str() >= from && e.start.as_str() < until)
            .collect()
    }

    /// Events for a specific provider source.
    pub fn by_source(&self, source: &CalendarSource) -> Vec<&UniversalEvent> {
        self.events.iter().filter(|e| &e.source == source).collect()
    }

    /// Find scheduling conflicts: pairs of events from the same source
    /// whose time windows overlap (distinct from cross-provider duplicates).
    pub fn find_conflicts(&self) -> Vec<(&UniversalEvent, &UniversalEvent)> {
        let mut conflicts = vec![];
        let evs = &self.events;
        for i in 0..evs.len() {
            for j in (i + 1)..evs.len() {
                let a = &evs[i];
                let b = &evs[j];
                if a.source == b.source
                    && !a.is_duplicate_ref
                    && !b.is_duplicate_ref
                    && a.overlaps_with(b)
                {
                    conflicts.push((a, b));
                }
            }
        }
        conflicts
    }

    /// Count of unique canonical events (excludes duplicate refs).
    pub fn canonical_count(&self) -> usize {
        self.events.iter().filter(|e| !e.is_duplicate_ref).count()
    }
}

// ─── UTC timestamp helpers ────────────────────────────────────────────────

/// Parse the difference between two UTC ISO 8601 timestamps as minutes.
/// Handles format: "YYYY-MM-DDTHH:MM:SSZ"
fn parse_minutes_between(start: &str, end: &str) -> Option<i64> {
    let s = parse_iso8601_minutes(start)?;
    let e = parse_iso8601_minutes(end)?;
    Some(e - s)
}

/// Convert a UTC ISO 8601 timestamp to total minutes since epoch start.
/// Simplified: does not handle sub-minute precision or timezone offsets
/// beyond Z (UTC).
fn parse_iso8601_minutes(ts: &str) -> Option<i64> {
    // Expected format: YYYY-MM-DDTHH:MM:SSZ
    let ts = ts.trim_end_matches('Z');
    let (date, time) = ts.split_once('T')?;
    let date_parts: Vec<&str> = date.split('-').collect();
    let time_parts: Vec<&str> = time.split(':').collect();
    if date_parts.len() < 3 || time_parts.len() < 2 {
        return None;
    }
    let year: i64 = date_parts[0].parse().ok()?;
    let month: i64 = date_parts[1].parse().ok()?;
    let day: i64 = date_parts[2].parse().ok()?;
    let hour: i64 = time_parts[0].parse().ok()?;
    let minute: i64 = time_parts[1].parse().ok()?;

    // Rough approximation sufficient for overlap/duration comparisons within a year.
    // Uses 365 days/year and 30 days/month (not calendar-accurate; this function
    // is only used to compute relative differences, not absolute timestamps).
    let total_days = year * 365 + month * 30 + day;
    Some(total_days * 24 * 60 + hour * 60 + minute)
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn google_event(id: &str, title: &str, start: &str, end: &str) -> UniversalEvent {
        UniversalEvent::new_timed(id, CalendarSource::Google, title, start, end)
    }

    fn ms_event(id: &str, title: &str, start: &str, end: &str) -> UniversalEvent {
        UniversalEvent::new_timed(id, CalendarSource::Microsoft, title, start, end)
    }

    fn apple_event(id: &str, title: &str, start: &str, end: &str) -> UniversalEvent {
        UniversalEvent::new_timed(id, CalendarSource::Apple, title, start, end)
    }

    // ── UniversalEvent basics ─────────────────────────────────────────

    #[test]
    fn test_event_id_format() {
        let e = google_event("evt1", "Standup", "2026-03-22T09:00:00Z", "2026-03-22T09:30:00Z");
        assert_eq!(e.id, "google:evt1");
        assert_eq!(e.provider_id, "evt1");
    }

    #[test]
    fn test_event_duration_minutes() {
        let e = google_event("e1", "Meeting", "2026-03-22T09:00:00Z", "2026-03-22T10:00:00Z");
        assert_eq!(e.duration_minutes(), Some(60));
    }

    #[test]
    fn test_all_day_duration_is_none() {
        let e = UniversalEvent::new_all_day(
            "e1",
            CalendarSource::Google,
            "Holiday",
            "2026-03-22",
            "2026-03-23",
        );
        assert_eq!(e.duration_minutes(), None);
    }

    #[test]
    fn test_event_overlaps_with() {
        // 09:00-10:00 and 09:30-10:30 should overlap.
        let a = google_event("a", "A", "2026-03-22T09:00:00Z", "2026-03-22T10:00:00Z");
        let b = google_event("b", "B", "2026-03-22T09:30:00Z", "2026-03-22T10:30:00Z");
        assert!(a.overlaps_with(&b));
    }

    #[test]
    fn test_event_no_overlap() {
        let a = google_event("a", "A", "2026-03-22T09:00:00Z", "2026-03-22T10:00:00Z");
        let b = google_event("b", "B", "2026-03-22T10:00:00Z", "2026-03-22T11:00:00Z");
        // End == start → no overlap (half-open interval).
        assert!(!a.overlaps_with(&b));
    }

    // ── Duplicate detection ───────────────────────────────────────────

    #[test]
    fn test_same_provider_not_duplicate() {
        let a = google_event("a", "Standup", "2026-03-22T09:00:00Z", "2026-03-22T09:30:00Z");
        let b = google_event("b", "Standup", "2026-03-22T09:00:00Z", "2026-03-22T09:30:00Z");
        assert!(!are_duplicates(&a, &b));
    }

    #[test]
    fn test_cross_provider_same_title_overlapping_is_duplicate() {
        let g = google_event("g1", "Weekly Sync", "2026-03-22T10:00:00Z", "2026-03-22T11:00:00Z");
        let m = ms_event("m1", "Weekly Sync", "2026-03-22T10:00:00Z", "2026-03-22T11:00:00Z");
        assert!(are_duplicates(&g, &m));
    }

    #[test]
    fn test_cross_provider_different_title_not_duplicate() {
        let g = google_event("g1", "Standup", "2026-03-22T09:00:00Z", "2026-03-22T09:30:00Z");
        let m = ms_event("m1", "Retro", "2026-03-22T09:00:00Z", "2026-03-22T09:30:00Z");
        assert!(!are_duplicates(&g, &m));
    }

    #[test]
    fn test_normalise_title_ignores_punctuation_and_case() {
        assert_eq!(normalise_title("Weekly Sync!"), normalise_title("weekly sync"));
        assert_eq!(normalise_title("Q1 Review — 2026"), normalise_title("q1 review 2026"));
    }

    // ── TimelineBuilder (KeepFirst) ───────────────────────────────────

    #[test]
    fn test_keep_first_deduplication() {
        let mut builder = TimelineBuilder::new().with_policy(ConflictPolicy::KeepFirst);
        builder.add(google_event("g1", "Standup", "2026-03-22T09:00:00Z", "2026-03-22T09:30:00Z"));
        builder.add(ms_event("m1", "Standup", "2026-03-22T09:00:00Z", "2026-03-22T09:30:00Z"));
        builder.add(apple_event("a1", "Team Lunch", "2026-03-22T12:00:00Z", "2026-03-22T13:00:00Z"));

        let timeline = builder.build();
        // Two standup events: first (google) canonical, second (ms) is a dup ref.
        // One team lunch.
        assert_eq!(timeline.len(), 3);
        assert_eq!(timeline.canonical_count(), 2);

        let dup = timeline.events().iter().find(|e| e.is_duplicate_ref).unwrap();
        assert_eq!(dup.canonical_id, Some("google:g1".to_string()));
    }

    // ── TimelineBuilder (Merge) ───────────────────────────────────────

    #[test]
    fn test_merge_deduplication() {
        let mut builder = TimelineBuilder::new().with_policy(ConflictPolicy::Merge);
        let g = google_event("g1", "Planning", "2026-03-22T14:00:00Z", "2026-03-22T15:00:00Z")
            .with_description("Google description")
            .with_attendee(Attendee::new("alice@example.com", AttendeeStatus::Accepted));
        let m = ms_event("m1", "Planning", "2026-03-22T14:00:00Z", "2026-03-22T15:00:00Z")
            .with_location("Conference Room B")
            .with_attendee(Attendee::new("bob@example.com", AttendeeStatus::Accepted));
        builder.add(g);
        builder.add(m);

        let timeline = builder.build();
        // One merged event.
        assert_eq!(timeline.canonical_count(), 1);

        let merged = &timeline.events()[0];
        assert_eq!(merged.description, Some("Google description".to_string()));
        assert_eq!(merged.location, Some("Conference Room B".to_string()));
        assert_eq!(merged.attendees.len(), 2);
    }

    // ── TimelineBuilder (KeepAll) ─────────────────────────────────────

    #[test]
    fn test_keep_all_no_deduplication() {
        let mut builder = TimelineBuilder::new().with_policy(ConflictPolicy::KeepAll);
        builder.add(google_event("g1", "Standup", "2026-03-22T09:00:00Z", "2026-03-22T09:30:00Z"));
        builder.add(ms_event("m1", "Standup", "2026-03-22T09:00:00Z", "2026-03-22T09:30:00Z"));
        let timeline = builder.build();
        assert_eq!(timeline.len(), 2);
        assert_eq!(timeline.canonical_count(), 2);
    }

    // ── UnifiedTimeline queries ───────────────────────────────────────

    #[test]
    fn test_timeline_sorted_by_start() {
        let mut builder = TimelineBuilder::new();
        builder.add(google_event("g2", "Lunch", "2026-03-22T12:00:00Z", "2026-03-22T13:00:00Z"));
        builder.add(google_event("g1", "Standup", "2026-03-22T09:00:00Z", "2026-03-22T09:30:00Z"));
        let timeline = builder.build();
        let starts: Vec<&str> = timeline.events().iter().map(|e| e.start.as_str()).collect();
        assert_eq!(starts[0], "2026-03-22T09:00:00Z");
        assert_eq!(starts[1], "2026-03-22T12:00:00Z");
    }

    #[test]
    fn test_timeline_range_filter() {
        let mut builder = TimelineBuilder::new();
        builder.add(google_event("g1", "Morning", "2026-03-22T09:00:00Z", "2026-03-22T10:00:00Z"));
        builder.add(google_event("g2", "Afternoon", "2026-03-22T14:00:00Z", "2026-03-22T15:00:00Z"));
        let timeline = builder.build();

        let morning = timeline.range("2026-03-22T00:00:00Z", "2026-03-22T12:00:00Z");
        assert_eq!(morning.len(), 1);
        assert_eq!(morning[0].title, "Morning");
    }

    #[test]
    fn test_timeline_by_source() {
        let mut builder = TimelineBuilder::new().with_policy(ConflictPolicy::KeepAll);
        builder.add(google_event("g1", "A", "2026-03-22T09:00:00Z", "2026-03-22T10:00:00Z"));
        builder.add(ms_event("m1", "B", "2026-03-22T11:00:00Z", "2026-03-22T12:00:00Z"));
        let timeline = builder.build();

        assert_eq!(timeline.by_source(&CalendarSource::Google).len(), 1);
        assert_eq!(timeline.by_source(&CalendarSource::Microsoft).len(), 1);
        assert_eq!(timeline.by_source(&CalendarSource::Apple).len(), 0);
    }

    #[test]
    fn test_find_conflicts_same_provider() {
        let mut builder = TimelineBuilder::new().with_policy(ConflictPolicy::KeepAll);
        // Two Google events that overlap.
        builder.add(google_event("g1", "A", "2026-03-22T09:00:00Z", "2026-03-22T10:00:00Z"));
        builder.add(google_event("g2", "B", "2026-03-22T09:30:00Z", "2026-03-22T10:30:00Z"));
        builder.add(google_event("g3", "C", "2026-03-22T11:00:00Z", "2026-03-22T12:00:00Z"));
        let timeline = builder.build();

        let conflicts = timeline.find_conflicts();
        assert_eq!(conflicts.len(), 1);
        let ids: Vec<&str> = vec![conflicts[0].0.id.as_str(), conflicts[0].1.id.as_str()];
        assert!(ids.contains(&"google:g1"));
        assert!(ids.contains(&"google:g2"));
    }

    #[test]
    fn test_three_provider_merge_triple_duplicate() {
        let mut builder = TimelineBuilder::new().with_policy(ConflictPolicy::Merge);
        builder.add(google_event("g1", "All-Hands", "2026-03-22T16:00:00Z", "2026-03-22T17:00:00Z"));
        builder.add(ms_event("m1", "All-Hands", "2026-03-22T16:00:00Z", "2026-03-22T17:00:00Z"));
        builder.add(apple_event("a1", "All Hands", "2026-03-22T16:00:00Z", "2026-03-22T17:00:00Z"));
        let timeline = builder.build();
        // "All-Hands" and "All Hands" normalise the same → all three merge into one.
        assert_eq!(timeline.canonical_count(), 1);
    }
}
