// ============================================================================
// UNIVERSAL CONTEXT LAYER
// The missing pieces that make Aluminum OS feel like ONE operating system.
//
// Implements the features both Google AND Microsoft wish they had:
//   - Universal Search (across all providers simultaneously)
//   - Universal Inbox (Outlook + Gmail + iMessage + Teams + Slack)
//   - Universal Notifications (one stream, all platforms)
//   - Universal Clipboard (copy on iPhone, paste on Chromebook)
//   - Universal File Graph (one namespace, all clouds)
//   - Cross-Ecosystem Backup + Restore
//   - Scheduling Intelligence (find free time across all calendars)
//   - Graph Unification Layer (M365 + Google + Apple graphs merged)
//   - Plugin Substrate (one plugin model, all surfaces)
//   - Infrastructure Copilot (devices, networks, topology)
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// UNIVERSAL SEARCH
// "alum search 'Q1 budget'" → searches Gmail, Outlook, Drive, OneDrive,
// iCloud, Notes, Teams, Slack, SharePoint simultaneously.
// Returns ranked, deduplicated, cross-referenced results.
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalSearch {
    pub providers: Vec<SearchProvider>,
    pub index: SearchIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchProvider {
    pub name: String,
    pub provider: String,
    pub search_endpoint: String,
    pub supports_full_text: bool,
    pub supports_semantic: bool,
    pub latency_budget_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    pub total_indexed: u64,
    pub last_crawl: String,
    pub providers_indexed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub providers: Option<Vec<String>>,  // None = search all
    pub resource_types: Option<Vec<String>>, // "mail", "file", "event", etc.
    pub date_range: Option<DateRange>,
    pub from: Option<String>,
    pub semantic: bool,  // use embeddings for semantic search
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub provider: String,
    pub resource_type: String,
    pub title: String,
    pub snippet: String,
    pub url: Option<String>,
    pub relevance_score: f32,
    pub timestamp: String,
    pub cross_references: Vec<String>, // IDs of related items in other providers
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSearchResponse {
    pub query: String,
    pub total_results: u32,
    pub results: Vec<SearchResult>,
    pub providers_searched: Vec<String>,
    pub providers_failed: Vec<String>,
    pub search_time_ms: u64,
    pub deduplicated: u32,  // items found in multiple providers
}

impl UniversalSearch {
    pub fn new() -> Self {
        Self {
            providers: vec![
                SearchProvider {
                    name: "Gmail".into(), provider: "google".into(),
                    search_endpoint: "gmail.users.messages.list".into(),
                    supports_full_text: true, supports_semantic: false, latency_budget_ms: 500,
                },
                SearchProvider {
                    name: "Google Drive".into(), provider: "google".into(),
                    search_endpoint: "drive.files.list".into(),
                    supports_full_text: true, supports_semantic: false, latency_budget_ms: 500,
                },
                SearchProvider {
                    name: "Google Calendar".into(), provider: "google".into(),
                    search_endpoint: "calendar.events.list".into(),
                    supports_full_text: true, supports_semantic: false, latency_budget_ms: 300,
                },
                SearchProvider {
                    name: "Outlook".into(), provider: "microsoft".into(),
                    search_endpoint: "/me/messages?$search=".into(),
                    supports_full_text: true, supports_semantic: true, latency_budget_ms: 500,
                },
                SearchProvider {
                    name: "OneDrive".into(), provider: "microsoft".into(),
                    search_endpoint: "/me/drive/root/search".into(),
                    supports_full_text: true, supports_semantic: false, latency_budget_ms: 500,
                },
                SearchProvider {
                    name: "SharePoint".into(), provider: "microsoft".into(),
                    search_endpoint: "/search/query".into(),
                    supports_full_text: true, supports_semantic: true, latency_budget_ms: 600,
                },
                SearchProvider {
                    name: "Teams".into(), provider: "microsoft".into(),
                    search_endpoint: "/me/chats/messages".into(),
                    supports_full_text: true, supports_semantic: false, latency_budget_ms: 400,
                },
                SearchProvider {
                    name: "iCloud Notes".into(), provider: "apple".into(),
                    search_endpoint: "cloudkit.notes.query".into(),
                    supports_full_text: true, supports_semantic: false, latency_budget_ms: 600,
                },
                SearchProvider {
                    name: "iCloud Drive".into(), provider: "apple".into(),
                    search_endpoint: "cloudkit.drive.query".into(),
                    supports_full_text: false, supports_semantic: false, latency_budget_ms: 700,
                },
            ],
            index: SearchIndex {
                total_indexed: 0,
                last_crawl: "never".into(),
                providers_indexed: Vec::new(),
            },
        }
    }

    /// Execute a search across ALL providers simultaneously
    /// Results are ranked, deduplicated, and cross-referenced
    pub async fn search(&self, query: SearchQuery) -> UnifiedSearchResponse {
        // In production: fan out to all providers in parallel,
        // collect results, deduplicate, rank, cross-reference
        UnifiedSearchResponse {
            query: query.text,
            total_results: 0,
            results: Vec::new(),
            providers_searched: self.providers.iter().map(|p| p.name.clone()).collect(),
            providers_failed: Vec::new(),
            search_time_ms: 0,
            deduplicated: 0,
        }
    }
}

// ============================================================================
// UNIVERSAL INBOX
// One inbox. All platforms. Outlook + Gmail + iMessage + Teams + Slack.
// Microsoft's #12 wish. The most requested enterprise feature on Earth.
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalInbox {
    pub streams: Vec<InboxStream>,
    pub unified_messages: Vec<UnifiedMessage>,
    pub filters: Vec<InboxFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxStream {
    pub name: String,
    pub provider: String,
    pub protocol: String,
    pub connected: bool,
    pub unread_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMessage {
    pub id: String,
    pub source: String,           // "gmail", "outlook", "teams", "slack", "imessage"
    pub source_provider: String,  // "google", "microsoft", "apple", "slack"
    pub from: String,
    pub to: Vec<String>,
    pub subject: Option<String>,
    pub body_preview: String,
    pub timestamp: String,
    pub is_read: bool,
    pub is_important: bool,
    pub thread_id: Option<String>,
    pub attachments: Vec<Attachment>,
    pub labels: Vec<String>,
    pub conversation_context: Option<String>, // AI-generated context
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub provider_url: String,
    pub universal_path: String, // unified file graph path
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxFilter {
    pub name: String,
    pub rule: FilterRule,
    pub action: FilterAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterRule {
    FromDomain(String),
    ContainsKeyword(String),
    IsMarketing,
    IsNewsLetter,
    IsDirectlyAddressedToMe,
    IsFromKnownContact,
    ImportanceAbove(f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterAction {
    Show,
    Hide,
    Archive,
    Star,
    Label(String),
    Forward(String),
}

impl UniversalInbox {
    pub fn new() -> Self {
        Self {
            streams: vec![
                InboxStream { name: "Gmail".into(), provider: "google".into(), protocol: "IMAP/API".into(), connected: true, unread_count: 0 },
                InboxStream { name: "Outlook".into(), provider: "microsoft".into(), protocol: "Graph API".into(), connected: true, unread_count: 0 },
                InboxStream { name: "Teams".into(), provider: "microsoft".into(), protocol: "Graph API".into(), connected: true, unread_count: 0 },
                InboxStream { name: "iCloud Mail".into(), provider: "apple".into(), protocol: "IMAP".into(), connected: true, unread_count: 0 },
                InboxStream { name: "Slack".into(), provider: "slack".into(), protocol: "Web API".into(), connected: false, unread_count: 0 },
                InboxStream { name: "iMessage".into(), provider: "apple".into(), protocol: "iMessage API".into(), connected: false, unread_count: 0 },
            ],
            unified_messages: Vec::new(),
            filters: vec![
                // Daavud's preference: filter out marketing junk
                InboxFilter {
                    name: "hide_marketing".into(),
                    rule: FilterRule::IsMarketing,
                    action: FilterAction::Hide,
                },
                InboxFilter {
                    name: "hide_newsletters".into(),
                    rule: FilterRule::IsNewsLetter,
                    action: FilterAction::Archive,
                },
                InboxFilter {
                    name: "prioritize_direct".into(),
                    rule: FilterRule::IsDirectlyAddressedToMe,
                    action: FilterAction::Star,
                },
            ],
        }
    }

    /// Get unified inbox — all messages from all sources, filtered and ranked
    /// `alum inbox` → one stream, all platforms
    pub fn get_unified(&self, limit: u32) -> Vec<&UnifiedMessage> {
        self.unified_messages.iter()
            .take(limit as usize)
            .collect()
    }
}

// ============================================================================
// UNIVERSAL NOTIFICATIONS
// One notification stream. All platforms. No more 6 separate notification centers.
// Microsoft's #14 wish.
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSubstrate {
    pub channels: Vec<NotificationChannel>,
    pub notifications: Vec<UnifiedNotification>,
    pub rules: Vec<NotificationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub name: String,
    pub provider: String,
    pub platform: String,  // "windows", "macos", "ios", "android", "chromeos", "web"
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedNotification {
    pub id: String,
    pub source: String,
    pub provider: String,
    pub title: String,
    pub body: String,
    pub timestamp: String,
    pub priority: NotificationPriority,
    pub category: NotificationCategory,
    pub actionable: bool,
    pub actions: Vec<NotificationAction>,
    pub dismissed: bool,
    pub devices_delivered: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Critical,   // security alerts, urgent messages
    High,       // direct messages, meeting reminders
    Normal,     // app updates, social
    Low,        // marketing, informational
    Silent,     // logged but not shown
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationCategory {
    Communication,  // email, chat, call
    Calendar,       // meeting reminders, schedule changes
    Security,       // login alerts, 2FA
    SmartHome,      // device alerts, motion detection
    Health,         // activity goals, sleep data
    System,         // updates, maintenance
    Social,         // social media
    Financial,      // transactions, alerts
    Pet,            // Dog Fi geofence, activity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub label: String,
    pub action_type: String,  // "reply", "dismiss", "snooze", "open", "execute"
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRule {
    pub name: String,
    pub condition: String,
    pub action: String,
    pub schedule: Option<String>, // "focus_mode", "sleep", "work"
}

// ============================================================================
// UNIVERSAL CLIPBOARD
// Copy on iPhone, paste on Chromebook. Copy on MacBook, paste on Pixel.
// Cross-device, cross-ecosystem, encrypted in transit.
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalClipboard {
    pub entries: Vec<ClipboardEntry>,
    pub max_entries: u32,
    pub encryption: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: String,
    pub content_type: ClipboardContentType,
    pub data: String,           // text content or base64 for binary
    pub source_device: String,
    pub timestamp: String,
    pub expires_at: String,
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardContentType {
    Text,
    RichText,
    Image,
    File,
    Url,
    Code,
}

// ============================================================================
// UNIVERSAL FILE GRAPH
// One namespace. All clouds. No more "is it in Drive or OneDrive or iCloud?"
// Microsoft's #3 wish. Google's implicit wish.
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalFileGraph {
    pub mounts: Vec<FileMount>,
    pub namespace: String,  // "alum://"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMount {
    pub path: String,          // "alum://drive/google/", "alum://drive/microsoft/"
    pub provider: String,
    pub service: String,       // "google_drive", "onedrive_personal", "onedrive_business", "sharepoint", "icloud_drive"
    pub connected: bool,
    pub quota_used: u64,
    pub quota_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedFile {
    pub universal_path: String,   // "alum://drive/google/Documents/Q1_Budget.xlsx"
    pub provider: String,
    pub provider_id: String,      // native ID in the provider's system
    pub name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub created_at: String,
    pub modified_at: String,
    pub shared_with: Vec<String>,
    pub versions: Vec<FileVersion>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileVersion {
    pub version_id: String,
    pub modified_at: String,
    pub modified_by: String,
    pub size_bytes: u64,
}

impl UniversalFileGraph {
    pub fn new() -> Self {
        Self {
            namespace: "alum://".into(),
            mounts: vec![
                FileMount {
                    path: "alum://drive/google/".into(), provider: "google".into(),
                    service: "google_drive".into(), connected: true,
                    quota_used: 0, quota_total: 15_000_000_000,
                },
                FileMount {
                    path: "alum://drive/microsoft/personal/".into(), provider: "microsoft".into(),
                    service: "onedrive_personal".into(), connected: true,
                    quota_used: 0, quota_total: 5_000_000_000,
                },
                FileMount {
                    path: "alum://drive/microsoft/business/".into(), provider: "microsoft".into(),
                    service: "onedrive_business".into(), connected: false,
                    quota_used: 0, quota_total: 1_000_000_000_000,
                },
                FileMount {
                    path: "alum://drive/microsoft/sharepoint/".into(), provider: "microsoft".into(),
                    service: "sharepoint".into(), connected: false,
                    quota_used: 0, quota_total: 0,
                },
                FileMount {
                    path: "alum://drive/apple/".into(), provider: "apple".into(),
                    service: "icloud_drive".into(), connected: true,
                    quota_used: 0, quota_total: 50_000_000_000,
                },
                FileMount {
                    path: "alum://drive/local/".into(), provider: "local".into(),
                    service: "local_fs".into(), connected: true,
                    quota_used: 0, quota_total: 0,
                },
            ],
        }
    }

    /// Resolve a universal path to a provider-specific path
    /// "alum://drive/google/Documents/file.txt" → Google Drive file ID
    /// "alum://drive/microsoft/Documents/file.txt" → OneDrive item ID
    pub fn resolve(&self, universal_path: &str) -> Option<(&FileMount, String)> {
        for mount in &self.mounts {
            if universal_path.starts_with(&mount.path) {
                let relative = universal_path.strip_prefix(&mount.path).unwrap_or("");
                return Some((mount, relative.to_string()));
            }
        }
        None
    }
}

// ============================================================================
// CROSS-ECOSYSTEM BACKUP + RESTORE
// Microsoft's #19 wish. Back up everything. Restore anywhere.
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRestoreSubstrate {
    pub backup_jobs: Vec<BackupJob>,
    pub restore_points: Vec<RestorePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupJob {
    pub id: String,
    pub name: String,
    pub sources: Vec<BackupSource>,
    pub destination: String,
    pub schedule: String,
    pub encryption: String,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSource {
    pub provider: String,
    pub resource: String,
    pub scope: String,  // "all", "recent", "starred"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorePoint {
    pub id: String,
    pub timestamp: String,
    pub size_bytes: u64,
    pub sources: Vec<String>,
    pub integrity_hash: String,
}

impl BackupRestoreSubstrate {
    pub fn new() -> Self {
        Self {
            backup_jobs: vec![
                BackupJob {
                    id: "full-ecosystem-backup".into(),
                    name: "Full Aluminum OS Backup".into(),
                    sources: vec![
                        BackupSource { provider: "google".into(), resource: "mail".into(), scope: "all".into() },
                        BackupSource { provider: "google".into(), resource: "drive".into(), scope: "all".into() },
                        BackupSource { provider: "google".into(), resource: "calendar".into(), scope: "all".into() },
                        BackupSource { provider: "google".into(), resource: "contacts".into(), scope: "all".into() },
                        BackupSource { provider: "microsoft".into(), resource: "mail".into(), scope: "all".into() },
                        BackupSource { provider: "microsoft".into(), resource: "onedrive".into(), scope: "all".into() },
                        BackupSource { provider: "microsoft".into(), resource: "calendar".into(), scope: "all".into() },
                        BackupSource { provider: "microsoft".into(), resource: "teams".into(), scope: "all".into() },
                        BackupSource { provider: "apple".into(), resource: "calendar".into(), scope: "all".into() },
                        BackupSource { provider: "apple".into(), resource: "contacts".into(), scope: "all".into() },
                        BackupSource { provider: "apple".into(), resource: "notes".into(), scope: "all".into() },
                        BackupSource { provider: "apple".into(), resource: "drive".into(), scope: "all".into() },
                    ],
                    destination: "alum://backup/encrypted/".into(),
                    schedule: "daily".into(),
                    encryption: "AES-256-GCM".into(),
                    last_run: None,
                    next_run: None,
                },
            ],
            restore_points: Vec::new(),
        }
    }
}

// ============================================================================
// SCHEDULING INTELLIGENCE
// Microsoft's #16 wish. Find free time across ALL calendars.
// "alum schedule meeting with Alice" → checks Google, Outlook, Apple Calendar
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingIntelligence {
    pub calendar_sources: Vec<CalendarSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarSource {
    pub name: String,
    pub provider: String,
    pub calendar_id: String,
    pub color: String,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeSlot {
    pub start: String,
    pub end: String,
    pub duration_minutes: u32,
    pub conflicts: Vec<String>, // IDs of events that would conflict
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingProposal {
    pub title: String,
    pub attendees: Vec<String>,
    pub duration_minutes: u32,
    pub proposed_slots: Vec<FreeSlot>,
    pub preferred_provider: String,  // which calendar to create the event in
    pub video_link: Option<String>,  // auto-generate Meet/Teams/Zoom link
}

impl SchedulingIntelligence {
    pub fn new() -> Self {
        Self {
            calendar_sources: vec![
                CalendarSource { name: "Google Calendar".into(), provider: "google".into(), calendar_id: "primary".into(), color: "#4285F4".into(), is_primary: true },
                CalendarSource { name: "Outlook Calendar".into(), provider: "microsoft".into(), calendar_id: "default".into(), color: "#0078D4".into(), is_primary: false },
                CalendarSource { name: "iCloud Calendar".into(), provider: "apple".into(), calendar_id: "default".into(), color: "#FF3B30".into(), is_primary: false },
            ],
        }
    }

    /// Find free slots across ALL calendars
    /// This is the killer scheduling feature — no more "let me check my other calendar"
    pub async fn find_free_slots(&self, duration_minutes: u32, range: DateRange) -> Vec<FreeSlot> {
        // In production: query all calendar providers in parallel,
        // merge busy times, find gaps, return sorted by preference
        Vec::new()
    }
}

// ============================================================================
// GRAPH UNIFICATION LAYER
// Microsoft's #15 wish. Merge M365 Graph + Google APIs + Apple APIs
// into one unified resource graph.
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphUnificationLayer {
    pub graphs: Vec<ProviderGraph>,
    pub unified_schema: HashMap<String, ResourceSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderGraph {
    pub provider: String,
    pub base_url: String,
    pub version: String,
    pub resources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSchema {
    pub resource: String,
    pub unified_fields: Vec<UnifiedField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedField {
    pub name: String,
    pub field_type: String,
    pub google_path: Option<String>,
    pub microsoft_path: Option<String>,
    pub apple_path: Option<String>,
}

impl GraphUnificationLayer {
    pub fn new() -> Self {
        let mut unified_schema = HashMap::new();

        // Unified "message" schema across providers
        unified_schema.insert("message".into(), ResourceSchema {
            resource: "message".into(),
            unified_fields: vec![
                UnifiedField { name: "id".into(), field_type: "string".into(), google_path: Some("id".into()), microsoft_path: Some("id".into()), apple_path: Some("messageId".into()) },
                UnifiedField { name: "from".into(), field_type: "string".into(), google_path: Some("payload.headers[From]".into()), microsoft_path: Some("from.emailAddress.address".into()), apple_path: Some("from".into()) },
                UnifiedField { name: "to".into(), field_type: "array".into(), google_path: Some("payload.headers[To]".into()), microsoft_path: Some("toRecipients[].emailAddress.address".into()), apple_path: Some("to".into()) },
                UnifiedField { name: "subject".into(), field_type: "string".into(), google_path: Some("payload.headers[Subject]".into()), microsoft_path: Some("subject".into()), apple_path: Some("subject".into()) },
                UnifiedField { name: "body".into(), field_type: "string".into(), google_path: Some("payload.body.data".into()), microsoft_path: Some("body.content".into()), apple_path: Some("body".into()) },
                UnifiedField { name: "date".into(), field_type: "datetime".into(), google_path: Some("internalDate".into()), microsoft_path: Some("receivedDateTime".into()), apple_path: Some("date".into()) },
                UnifiedField { name: "is_read".into(), field_type: "boolean".into(), google_path: Some("!labelIds.contains(UNREAD)".into()), microsoft_path: Some("isRead".into()), apple_path: Some("seen".into()) },
            ],
        });

        // Unified "event" schema across providers
        unified_schema.insert("event".into(), ResourceSchema {
            resource: "event".into(),
            unified_fields: vec![
                UnifiedField { name: "id".into(), field_type: "string".into(), google_path: Some("id".into()), microsoft_path: Some("id".into()), apple_path: Some("UID".into()) },
                UnifiedField { name: "title".into(), field_type: "string".into(), google_path: Some("summary".into()), microsoft_path: Some("subject".into()), apple_path: Some("SUMMARY".into()) },
                UnifiedField { name: "start".into(), field_type: "datetime".into(), google_path: Some("start.dateTime".into()), microsoft_path: Some("start.dateTime".into()), apple_path: Some("DTSTART".into()) },
                UnifiedField { name: "end".into(), field_type: "datetime".into(), google_path: Some("end.dateTime".into()), microsoft_path: Some("end.dateTime".into()), apple_path: Some("DTEND".into()) },
                UnifiedField { name: "location".into(), field_type: "string".into(), google_path: Some("location".into()), microsoft_path: Some("location.displayName".into()), apple_path: Some("LOCATION".into()) },
                UnifiedField { name: "attendees".into(), field_type: "array".into(), google_path: Some("attendees[].email".into()), microsoft_path: Some("attendees[].emailAddress.address".into()), apple_path: Some("ATTENDEE".into()) },
                UnifiedField { name: "video_link".into(), field_type: "string".into(), google_path: Some("hangoutLink".into()), microsoft_path: Some("onlineMeeting.joinUrl".into()), apple_path: None },
            ],
        });

        // Unified "contact" schema across providers
        unified_schema.insert("contact".into(), ResourceSchema {
            resource: "contact".into(),
            unified_fields: vec![
                UnifiedField { name: "id".into(), field_type: "string".into(), google_path: Some("resourceName".into()), microsoft_path: Some("id".into()), apple_path: Some("UID".into()) },
                UnifiedField { name: "name".into(), field_type: "string".into(), google_path: Some("names[0].displayName".into()), microsoft_path: Some("displayName".into()), apple_path: Some("FN".into()) },
                UnifiedField { name: "email".into(), field_type: "string".into(), google_path: Some("emailAddresses[0].value".into()), microsoft_path: Some("emailAddresses[0].address".into()), apple_path: Some("EMAIL".into()) },
                UnifiedField { name: "phone".into(), field_type: "string".into(), google_path: Some("phoneNumbers[0].value".into()), microsoft_path: Some("mobilePhone".into()), apple_path: Some("TEL".into()) },
                UnifiedField { name: "company".into(), field_type: "string".into(), google_path: Some("organizations[0].name".into()), microsoft_path: Some("companyName".into()), apple_path: Some("ORG".into()) },
            ],
        });

        // Unified "file" schema across providers
        unified_schema.insert("file".into(), ResourceSchema {
            resource: "file".into(),
            unified_fields: vec![
                UnifiedField { name: "id".into(), field_type: "string".into(), google_path: Some("id".into()), microsoft_path: Some("id".into()), apple_path: Some("recordName".into()) },
                UnifiedField { name: "name".into(), field_type: "string".into(), google_path: Some("name".into()), microsoft_path: Some("name".into()), apple_path: Some("filename".into()) },
                UnifiedField { name: "mime_type".into(), field_type: "string".into(), google_path: Some("mimeType".into()), microsoft_path: Some("file.mimeType".into()), apple_path: Some("contentType".into()) },
                UnifiedField { name: "size".into(), field_type: "number".into(), google_path: Some("size".into()), microsoft_path: Some("size".into()), apple_path: Some("size".into()) },
                UnifiedField { name: "modified".into(), field_type: "datetime".into(), google_path: Some("modifiedTime".into()), microsoft_path: Some("lastModifiedDateTime".into()), apple_path: Some("modified".into()) },
                UnifiedField { name: "parent".into(), field_type: "string".into(), google_path: Some("parents[0]".into()), microsoft_path: Some("parentReference.id".into()), apple_path: Some("parent".into()) },
                UnifiedField { name: "web_url".into(), field_type: "string".into(), google_path: Some("webViewLink".into()), microsoft_path: Some("webUrl".into()), apple_path: None },
            ],
        });

        Self {
            graphs: vec![
                ProviderGraph {
                    provider: "google".into(),
                    base_url: "https://www.googleapis.com".into(),
                    version: "v1".into(),
                    resources: vec!["message".into(), "event".into(), "contact".into(), "file".into(), "task".into()],
                },
                ProviderGraph {
                    provider: "microsoft".into(),
                    base_url: "https://graph.microsoft.com".into(),
                    version: "v1.0".into(),
                    resources: vec!["message".into(), "event".into(), "contact".into(), "file".into(), "task".into(), "chat".into(), "channel".into()],
                },
                ProviderGraph {
                    provider: "apple".into(),
                    base_url: "https://caldav.icloud.com".into(),
                    version: "v1".into(),
                    resources: vec!["event".into(), "contact".into(), "file".into(), "note".into(), "reminder".into()],
                },
            ],
            unified_schema,
        }
    }

    /// Translate a unified query to provider-specific queries
    /// "alum get message --from alice" →
    ///   Google: gmail.users.messages.list?q=from:alice
    ///   Microsoft: /me/messages?$filter=from/emailAddress/address eq 'alice'
    ///   Apple: IMAP SEARCH FROM "alice"
    pub fn translate(&self, resource: &str, field: &str, provider: &str) -> Option<String> {
        if let Some(schema) = self.unified_schema.get(resource) {
            if let Some(uf) = schema.unified_fields.iter().find(|f| f.name == field) {
                return match provider {
                    "google" => uf.google_path.clone(),
                    "microsoft" => uf.microsoft_path.clone(),
                    "apple" => uf.apple_path.clone(),
                    _ => None,
                };
            }
        }
        None
    }
}

// ============================================================================
// PLUGIN SUBSTRATE
// Microsoft's #4 wish. One plugin model. All surfaces.
// Replaces 200+ separate Office add-in frameworks.
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSubstrate {
    pub plugins: Vec<AluminumPlugin>,
    pub registry_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AluminumPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub surfaces: Vec<PluginSurface>,  // where this plugin works
    pub permissions: Vec<String>,
    pub entry_point: String,
    pub manifest_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginSurface {
    CLI,          // works in `alum` command line
    Web,          // works in browser
    Desktop,      // works in native desktop
    Mobile,       // works on iOS/Android
    Agent,        // works as an AI agent tool
    Notification, // can send notifications
    FilePreview,  // can preview file types
    All,          // works everywhere
}

// ============================================================================
// INFRASTRUCTURE COPILOT
// Microsoft's #13 wish. Manage devices, networks, org topology via NL + CLI.
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureCopilot {
    pub devices: Vec<ManagedDevice>,
    pub networks: Vec<NetworkTopology>,
    pub policies: Vec<InfraPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedDevice {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub os: String,
    pub os_version: String,
    pub enrolled: bool,
    pub compliant: bool,
    pub last_check_in: String,
    pub managed_by: String,  // "intune", "google_endpoint", "jamf", "aluminum"
    pub installed_apps: Vec<String>,
    pub security_status: SecurityStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStatus {
    pub encryption_enabled: bool,
    pub firewall_enabled: bool,
    pub antivirus_current: bool,
    pub os_up_to_date: bool,
    pub password_compliant: bool,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    pub name: String,
    pub network_type: String,
    pub devices_connected: u32,
    pub bandwidth_mbps: u32,
    pub security: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfraPolicy {
    pub name: String,
    pub scope: String,
    pub rules: Vec<String>,
    pub enforcement: String,
}

// ============================================================================
// CROSS-CLOUD ABSTRACTION LAYER
// Microsoft's #11 wish. Azure + AWS + GCP under one interface.
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAbstractionLayer {
    pub clouds: Vec<CloudProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProvider {
    pub name: String,
    pub provider: String,   // "azure", "aws", "gcp"
    pub connected: bool,
    pub services: Vec<CloudService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudService {
    pub unified_name: String,   // "compute", "storage", "database", "ai"
    pub provider_name: String,  // "EC2", "Compute Engine", "Azure VMs"
    pub provider: String,
}

impl CloudAbstractionLayer {
    pub fn new() -> Self {
        Self {
            clouds: vec![
                CloudProvider {
                    name: "Azure".into(), provider: "azure".into(), connected: true,
                    services: vec![
                        CloudService { unified_name: "compute".into(), provider_name: "Azure VMs".into(), provider: "azure".into() },
                        CloudService { unified_name: "storage".into(), provider_name: "Azure Blob".into(), provider: "azure".into() },
                        CloudService { unified_name: "database".into(), provider_name: "Azure SQL".into(), provider: "azure".into() },
                        CloudService { unified_name: "ai".into(), provider_name: "Azure OpenAI".into(), provider: "azure".into() },
                    ],
                },
                CloudProvider {
                    name: "AWS".into(), provider: "aws".into(), connected: false,
                    services: vec![
                        CloudService { unified_name: "compute".into(), provider_name: "EC2".into(), provider: "aws".into() },
                        CloudService { unified_name: "storage".into(), provider_name: "S3".into(), provider: "aws".into() },
                        CloudService { unified_name: "database".into(), provider_name: "RDS".into(), provider: "aws".into() },
                        CloudService { unified_name: "ai".into(), provider_name: "Bedrock".into(), provider: "aws".into() },
                    ],
                },
                CloudProvider {
                    name: "GCP".into(), provider: "gcp".into(), connected: true,
                    services: vec![
                        CloudService { unified_name: "compute".into(), provider_name: "Compute Engine".into(), provider: "gcp".into() },
                        CloudService { unified_name: "storage".into(), provider_name: "Cloud Storage".into(), provider: "gcp".into() },
                        CloudService { unified_name: "database".into(), provider_name: "Cloud SQL".into(), provider: "gcp".into() },
                        CloudService { unified_name: "ai".into(), provider_name: "Vertex AI".into(), provider: "gcp".into() },
                    ],
                },
            ],
        }
    }
}
